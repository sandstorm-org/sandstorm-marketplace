//! Convert verified package metadata without reading or writing registry files.
use crate::{ids, package_capnp as p};
use anyhow::{Context, Result, bail};
use marketplace_model::app_index::{AppDetailsForMarket, AppSummary, Screenshot};
use sha2::Digest;
use std::path::Path;

/// Display overrides carried over from the original marketplace. Its indexer
/// applied /var/descriptions after reading signed metadata (see
/// sandstorm/app-index/indexer.c++ and ShortDescriptionOverrides in the schema).
/// These four differences were confirmed against the exported public catalog;
/// Let's Chat and MediaGoblin predate the required manifest field.
/// Never rewrite the verified metadata or the signed SPK itself.
pub(crate) fn short_description<'a>(info: p::verified_info::Reader<'a>) -> Result<&'a str> {
    let original = info
        .get_metadata()?
        .get_short_description()?
        .get_default_text()?
        .to_str()?;
    let app_id = ids::app_id(&ids::app_bytes(info.get_app_id()?));
    Ok(match app_id.as_str() {
        "qkgkaxfqhgsff8zgx2f4nf1a8xvmpte6wa19egmfkk06mzt7e8dh" => "Chat room",
        "70awyqss6jq2gkz7dwzsnvumzr07256pzdt3hda9acfuxwh6uh7h" => "Media sharing",
        "nn7axgy3y8kvd0m1mtk3cwca34t916p5d7m4j1j2e874nuz3t8y0" => "File sharing",
        "vxe8awcxvtj6yu0vgjpm1tsaeu7x8v8tfp71tyvnm6ykkephu9q0" => "Note-taking",
        _ => original,
    })
}

pub(crate) struct Entry<'a> {
    pub app: AppSummary,
    pub details: AppDetailsForMarket,
    pub fingerprint: &'a str,
    assets: Vec<Asset<'a>>,
}
struct Asset<'a> {
    name: String,
    data: &'a [u8],
}
impl Asset<'_> {
    fn data_url(&self) -> String {
        use base64::Engine;
        let mime = if self.name.ends_with(".svg") {
            "image/svg+xml"
        } else if self.name.ends_with(".png") {
            "image/png"
        } else {
            "image/jpeg"
        };
        format!(
            "data:{mime};base64,{}",
            base64::engine::general_purpose::STANDARD.encode(self.data)
        )
    }
}
impl Entry<'_> {
    pub fn icon_data_url(&self) -> Option<String> {
        self.assets
            .iter()
            .find(|asset| asset.name == self.app.image_id)
            .map(Asset::data_url)
    }
    pub fn image_data_urls(&self) -> std::collections::BTreeMap<String, String> {
        self.assets
            .iter()
            .map(|asset| (asset.name.clone(), asset.data_url()))
            .collect()
    }

    pub fn write_assets(&self, catalog: &Path, previous: &Path) -> Result<()> {
        for asset in &self.assets {
            let path = catalog.join("images").join(&asset.name);
            if !path.exists() {
                // Content-addressed images are immutable. Reuse their durable
                // bytes without dirtying them again on every full catalog build.
                let existing = previous.join("images").join(&asset.name);
                if std::fs::symlink_metadata(&existing)
                    .is_ok_and(|metadata| metadata.file_type().is_file())
                    && std::fs::hard_link(&existing, &path).is_ok()
                {
                    continue;
                }
                std::fs::write(&path, asset.data)?;
            }
        }
        Ok(())
    }
}
/// Read only the icon, without hashing screenshots or parsing the rest of the listing.
pub(crate) fn icon(info: p::verified_info::Reader<'_>) -> Result<Option<(&str, &[u8])>> {
    let icons = info.get_metadata()?.get_icons();
    let icon = if icons.has_market() {
        icons.get_market()?
    } else if icons.has_app_grid() {
        icons.get_app_grid()?
    } else {
        return Ok(None);
    };
    Ok(match icon.which()? {
        p::metadata::icon::Svg(svg) => Some(("image/svg+xml", svg?.to_str()?.as_bytes())),
        p::metadata::icon::Png(png) => Some((
            "image/png",
            if png.has_dpi1x() {
                png.get_dpi1x()?
            } else {
                png.get_dpi2x()?
            },
        )),
        _ => None,
    })
}

pub(crate) fn parse<'a>(info: p::verified_info::Reader<'a>, published: i64) -> Result<Entry<'a>> {
    parse_listing(info, published, true)
}

/// Review tables need text and an icon, not screenshot assets or their hashes.
pub(crate) fn parse_for_review<'a>(
    info: p::verified_info::Reader<'a>,
    published: i64,
) -> Result<Entry<'a>> {
    parse_listing(info, published, false)
}

fn parse_listing<'a>(
    info: p::verified_info::Reader<'a>,
    published: i64,
    screenshots_needed: bool,
) -> Result<Entry<'a>> {
    let meta = info.get_metadata()?;
    let icons = meta.get_icons();
    let mut assets = Vec::new();
    let mut collect_image = |data: &'a [u8], extension| -> String {
        let hash = blake2::Blake2b::<blake2::digest::consts::U16>::digest(data);
        let name = format!("{}.{}", hex::encode(hash), extension);
        assets.push(Asset {
            name: name.clone(),
            data,
        });
        name
    };
    let mut image_id = String::new();
    if icons.has_market() || icons.has_app_grid() {
        let icon = if icons.has_market() {
            icons.get_market()?
        } else {
            icons.get_app_grid()?
        };
        image_id = match icon.which()? {
            p::metadata::icon::Svg(svg) => collect_image(svg?.to_str()?.as_bytes(), "svg"),
            p::metadata::icon::Png(png) => collect_image(
                if png.has_dpi1x() {
                    png.get_dpi1x()?
                } else {
                    png.get_dpi2x()?
                },
                "png",
            ),
            _ => String::new(),
        };
    }
    let mut screenshots = Vec::new();
    if screenshots_needed {
        for shot in meta.get_screenshots()? {
            let image_id = match shot.which()? {
                p::metadata::screenshot::Png(png) => collect_image(png?, "png"),
                p::metadata::screenshot::Jpeg(jpeg) => collect_image(jpeg?, "jpeg"),
                _ => bail!("unknown screenshot format"),
            };
            screenshots.push(Screenshot {
                image_id,
                width: shot.get_width(),
                height: shot.get_height(),
            });
        }
    }
    let (is_open_source, license) = match meta.get_license().which()? {
        p::metadata::license::OpenSource(value) => (
            true,
            LICENSES
                .get(value? as usize)
                .unwrap_or(&"Unknown")
                .to_string(),
        ),
        p::metadata::license::Proprietary(_) => (false, "Proprietary".to_string()),
        p::metadata::license::PublicDomain(_) => (false, "Public Domain".to_string()),
        _ => (false, String::new()),
    };
    Ok(Entry {
        app: AppSummary {
            deprecation: None,
            app_id: ids::app_id(&ids::app_bytes(info.get_app_id()?)),
            package_id: hex::encode(ids::package_bytes(info.get_package_id()?)),
            name: info.get_title()?.get_default_text()?.to_str()?.to_string(),
            version: info
                .get_marketing_version()?
                .get_default_text()?
                .to_str()?
                .to_string(),
            version_number: info.get_version(),
            image_id,
            web_link: meta.get_website()?.to_str()?.to_string(),
            code_link: meta.get_code_url()?.to_str()?.to_string(),
            is_open_source,
            categories: meta
                .get_categories()?
                .iter()
                .map(|c| c.map(|v| CATEGORIES[v as usize].to_string()))
                .collect::<std::result::Result<_, _>>()?,
            author: None,
            author_pgp_key_fingerprint: info
                .get_author_pgp_key_fingerprint()?
                .to_str()?
                .to_string(),
            upstream_author: meta
                .get_author()
                .get_upstream_author()?
                .to_str()?
                .to_string(),
            short_description: short_description(info)?.to_owned(),
            created_at: chrono::DateTime::from_timestamp(published, 0)
                .context("invalid publish date")?
                .format("%Y-%m-%dT%H:%M:%SZ")
                .to_string(),
        },
        details: AppDetailsForMarket {
            description: meta
                .get_description()?
                .get_default_text()?
                .to_str()?
                .to_string(),
            change_log: meta
                .get_change_log()?
                .get_default_text()?
                .to_str()?
                .to_string(),
            screenshots,
            license,
        },
        fingerprint: info.get_author_pgp_key_fingerprint()?.to_str()?,
        assets,
    })
}
const CATEGORIES: &[&str] = &[
    "Other",
    "Productivity",
    "Communications",
    "Social",
    "Web Publishing",
    "Office",
    "DevTools",
    "Science",
    "Media",
    "Games",
    "Graphics",
];
const LICENSES: &[&str] = &[
    "Invalid",
    "MIT License",
    "Apache License v2",
    "GNU GPL v3",
    "GNU AGPL v3",
    "BSD 3-Clause",
    "BSD 2-Clause",
    "GNU GPL v2",
    "GNU LGPL v2.1",
    "GNU LGPL v3",
    "ISC License",
    "Artistic License v2",
    "Python License v2",
    "PHP License v3",
    "Mozilla Public License v2",
    "CDDL",
    "Eclipse Public License",
    "Common Public Attribution License",
    "Zlib/libpng License",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_descriptions_apply_to_listings_without_changing_verified_metadata() -> Result<()> {
        for (app_id, original, expected) in [
            (
                "qkgkaxfqhgsff8zgx2f4nf1a8xvmpte6wa19egmfkk06mzt7e8dh",
                "",
                "Chat room",
            ),
            (
                "70awyqss6jq2gkz7dwzsnvumzr07256pzdt3hda9acfuxwh6uh7h",
                "",
                "Media sharing",
            ),
            (
                "nn7axgy3y8kvd0m1mtk3cwca34t916p5d7m4j1j2e874nuz3t8y0",
                "Drag-n-drop file sharing",
                "File sharing",
            ),
            (
                "vxe8awcxvtj6yu0vgjpm1tsaeu7x8v8tfp71tyvnm6ykkephu9q0",
                "Notetaking",
                "Note-taking",
            ),
        ] {
            let mut message = capnp::message::Builder::new_default();
            let mut info = message.init_root::<p::verified_info::Builder>();
            ids::set_app(info.reborrow().init_app_id(), &ids::parse_app_id(app_id)?);
            info.init_metadata()
                .init_short_description()
                .set_default_text(original);
            let before = crate::database::encode(&message)?;
            let info = message.get_root_as_reader()?;
            assert_eq!(short_description(info)?, expected);
            assert_eq!(parse(info, 0)?.app.short_description, expected);
            assert_eq!(crate::database::encode(&message)?, before);

            // A different signing identity cannot inherit an allowlisted app's
            // description, even with identical (or missing) manifest text.
            ids::set_app(
                message
                    .get_root::<p::verified_info::Builder>()?
                    .init_app_id(),
                &[42; 32],
            );
            let info = message.get_root_as_reader()?;
            assert_eq!(short_description(info)?, original);
            assert_eq!(parse(info, 0)?.app.short_description, original);
        }
        Ok(())
    }

    #[test]
    fn parsed_assets_are_only_written_when_publishing() -> Result<()> {
        let mut message = capnp::message::Builder::new_default();
        let mut info = message.init_root::<p::verified_info::Builder>();
        let mut metadata = info.reborrow().init_metadata();
        metadata
            .reborrow()
            .init_icons()
            .init_market()
            .set_svg("<svg/>");
        let mut shot = metadata.init_screenshots(1).get(0);
        shot.set_png(b"screenshot");
        shot.set_width(640);
        shot.set_height(480);
        let entry = parse(message.get_root_as_reader()?, 0)?;
        let output = tempfile::tempdir()?;
        std::fs::create_dir(output.path().join("images"))?;
        assert_eq!(std::fs::read_dir(output.path().join("images"))?.count(), 0);
        assert_eq!(entry.details.screenshots[0].width, 640);
        entry.write_assets(output.path(), output.path())?;
        assert_eq!(
            std::fs::read(output.path().join("images").join(&entry.app.image_id))?,
            b"<svg/>"
        );
        assert_eq!(
            std::fs::read(
                output
                    .path()
                    .join("images")
                    .join(&entry.details.screenshots[0].image_id)
            )?,
            b"screenshot"
        );
        let next = tempfile::tempdir()?;
        std::fs::create_dir(next.path().join("images"))?;
        entry.write_assets(next.path(), output.path())?;
        use std::os::unix::fs::MetadataExt;
        let old = output.path().join("images").join(&entry.app.image_id);
        let reused = next.path().join("images").join(&entry.app.image_id);
        assert_eq!(
            std::fs::metadata(old)?.ino(),
            std::fs::metadata(reused)?.ino()
        );
        Ok(())
    }
}
