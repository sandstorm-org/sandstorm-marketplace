use crate::registry::Registry;
use anyhow::{Result, ensure};
use marketplace_model::branding::Branding;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub revision: i64,
    pub branding: Branding,
    pub header_style: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Update {
    pub revision: i64,
    pub branding: Branding,
}

impl Registry {
    pub(crate) fn branding_settings(&self) -> Result<Settings> {
        self.database.read(|db| {
            let (revision, json): (i64, Option<String>) = db.query_row(
                "SELECT revision,settings_json FROM branding_settings WHERE singleton=1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?;
            let branding: Branding = json
                .map(|json| serde_json::from_str(&json))
                .transpose()?
                .unwrap_or_default();
            Ok(Settings {
                revision,
                header_style: branding.header_style(),
                branding,
            })
        })
    }

    /// Caller holds the operation gate and has approver permission.
    pub(crate) fn update_branding(&self, mut update: Update) -> Result<()> {
        update.branding.validate()?;
        let json = serde_json::to_string(&update.branding)?;
        self.database.transaction(|db| {
            let (revision, previous): (i64, Option<String>) = db.query_row(
                "SELECT revision,settings_json FROM branding_settings WHERE singleton=1", [],
                |row| Ok((row.get(0)?, row.get(1)?)))?;
            // Retry a failed publication without committing the same edit again.
            if update.revision.checked_add(1) == Some(revision) && previous.as_deref() == Some(&json) { return Ok(()); }
            ensure!(revision == update.revision, "appearance changed; reload before saving");
            db.execute("UPDATE branding_settings SET settings_json=?,revision=revision+1 WHERE singleton=1", [json])?;
            db.execute("UPDATE registry_state SET state_revision=state_revision+1", [])?;
            Ok(())
        })?;
        self.publish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn persists_isolated_settings_and_retries_failed_publication() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let registry = Registry::new(dir.path().into())?;
        assert_eq!(registry.branding_settings()?.branding, Branding::default());
        let update = || Update {
            revision: 0,
            branding: Branding {
                title: "Our <Market>".into(),
                header_color: "#207080".into(),
                show_subheader: false,
            },
        };
        std::fs::write(dir.path().join("catalog"), "blocked")?;
        assert!(registry.update_branding(update()).is_err());
        assert_eq!(registry.branding_settings()?.revision, 1);
        std::fs::remove_file(dir.path().join("catalog"))?;
        registry.update_branding(update())?;
        assert_eq!(registry.branding_settings()?.revision, 1);
        let html = std::fs::read_to_string(dir.path().join("www/index.html"))?;
        assert!(html.contains("Our &#60;Market&#62;") || html.contains("Our &lt;Market&gt;"));
        assert!(!html.contains("<title>Our <Market>"));
        drop(registry);
        let registry = Registry::new(dir.path().into())?;
        assert_eq!(registry.branding_settings()?.branding, update().branding);
        assert!(
            registry
                .update_branding(Update {
                    revision: 0,
                    branding: Branding::default()
                })
                .is_err()
        );
        let other = tempfile::tempdir()?;
        assert_eq!(
            Registry::new(other.path().into())?
                .branding_settings()?
                .branding,
            Branding::default()
        );
        Ok(())
    }
}
