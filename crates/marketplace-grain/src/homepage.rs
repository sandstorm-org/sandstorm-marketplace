use crate::{
    ids,
    registry::{Registry, ReviewRelease},
};
use anyhow::{Result, ensure};
use marketplace_model::homepage::HomepageOrder;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Serialize)]
pub struct Settings {
    pub revision: i64,
    pub pinned: Vec<String>,
    pub hidden: Vec<String>,
    pub order: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Update {
    pub revision: i64,
    pub hidden: Vec<String>,
    pub pinned: Vec<String>,
}

impl Registry {
    pub(crate) fn homepage_order(&self) -> Result<(i64, HomepageOrder)> {
        self.database.read(|db| {
            let (revision, json): (i64, Option<String>) = db.query_row(
                "SELECT revision,order_json FROM homepage_settings WHERE singleton=1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            Ok((
                revision,
                json.map(|json| serde_json::from_str(&json))
                    .transpose()?
                    .unwrap_or_default(),
            ))
        })
    }

    pub(crate) fn homepage_settings(&self, releases: &[ReviewRelease]) -> Result<Settings> {
        let (revision, order) = self.homepage_order()?;
        let mut apps = BTreeMap::new();
        for release in releases {
            if !release.current || !release.public_page || release.app.deprecation.is_some() {
                continue;
            }
            let app = &release.app;
            if apps.get(&app.app_id).is_none_or(
                |old: &&marketplace_model::app_index::AppSummary| {
                    old.version_number < app.version_number
                },
            ) {
                apps.insert(app.app_id.clone(), app);
            }
        }
        let mut apps: Vec<_> = apps.into_values().cloned().collect();
        HomepageOrder::default().sort(&mut apps);
        Ok(Settings {
            revision,
            pinned: order
                .pinned
                .into_iter()
                .filter(|id| apps.iter().any(|app| &app.app_id == id))
                .collect(),
            hidden: order
                .hidden
                .into_iter()
                .filter(|id| apps.iter().any(|app| &app.app_id == id))
                .collect(),
            order: apps.into_iter().map(|a| a.app_id).collect(),
        })
    }

    /// Caller holds the registry operation gate and has approver permission.
    pub(crate) fn update_homepage(&self, update: Update) -> Result<()> {
        ensure!(
            update.pinned.len() + update.hidden.len() <= 10000,
            "too many homepage entries"
        );
        let mut seen = BTreeSet::new();
        for id in update.pinned.iter().chain(update.hidden.iter()) {
            ids::parse_app_id(id)?;
            ensure!(seen.insert(id), "duplicate app in homepage order");
        }
        self.database.transaction(|db| {
            for id in update.pinned.iter().chain(update.hidden.iter()) {
                ensure!(
                    db.query_row(
                        "SELECT EXISTS(SELECT 1 FROM apps WHERE app_id=?)",
                        [id],
                        |r| r.get::<_, bool>(0)
                    )?,
                    "unknown app in homepage order"
                );
            }
            let json = serde_json::to_string(&HomepageOrder {
                pinned: update.pinned,
                hidden: update.hidden,
            })?;
            let (revision, previous): (i64, Option<String>) = db.query_row(
                "SELECT revision,order_json FROM homepage_settings WHERE singleton=1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?;
            // A publication failure or lost response must allow the same save to retry.
            // Other stale edits still fail, even if an intervening edit restored this value.
            if update.revision.checked_add(1) == Some(revision)
                && previous.as_deref() == Some(&json)
            {
                return Ok(());
            }
            ensure!(
                revision == update.revision,
                "homepage order changed; reload before saving"
            );
            db.execute(
                "UPDATE homepage_settings SET order_json=?,revision=revision+1 WHERE singleton=1",
                [json],
            )?;
            db.execute(
                "UPDATE registry_state SET state_revision=state_revision+1",
                [],
            )?;
            Ok(())
        })?;
        self.publish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identical_retry_recovers_failed_publication_without_another_revision() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let registry = Registry::new(dir.path().into())?;
        std::fs::write(dir.path().join("catalog"), "blocked")?;
        let update = || Update {
            revision: 0,
            pinned: vec![],
            hidden: vec![],
        };
        assert!(registry.update_homepage(update()).is_err());
        assert_eq!(registry.homepage_order()?.0, 1);
        std::fs::remove_file(dir.path().join("catalog"))?;
        registry.update_homepage(update())?;
        assert_eq!(registry.homepage_order()?.0, 1);
        assert!(dir.path().join("www/index.html").is_file());
        Ok(())
    }

    #[test]
    fn settings_survive_restart_and_reject_stale_or_unknown_entries() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let registry = Registry::new(dir.path().into())?;
        assert!(registry.homepage_order()?.1.pinned.is_empty());
        registry.update_homepage(Update {
            revision: 0,
            pinned: vec![],
            hidden: vec![],
        })?;
        drop(registry);
        let registry = Registry::new(dir.path().into())?;
        assert_eq!(registry.homepage_order()?.0, 1);
        assert!(registry.homepage_order()?.1.pinned.is_empty());
        registry.update_homepage(Update {
            revision: 1,
            pinned: vec![],
            hidden: vec![],
        })?;
        assert!(
            registry
                .update_homepage(Update {
                    revision: 0,
                    pinned: vec![],
                    hidden: vec![]
                })
                .is_err()
        );
        assert!(
            registry
                .update_homepage(Update {
                    revision: 1,
                    pinned: vec![],
                    hidden: vec![marketplace_model::homepage::LEGACY_ORDER[0].into()]
                })
                .is_err()
        );
        assert_eq!(registry.homepage_order()?.0, 2);
        Ok(())
    }
}
