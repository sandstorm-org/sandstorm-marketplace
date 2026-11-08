# Changelog

## 0.5.1 — 2026-11-08

- Automatically reject earlier, lower-build pending releases when a newer release
  is approved, including withdrawn releases. Preserve the author's withdrawal.
- Default the header color to charcoal (#222222), preserving saved colors.
- Give the color picker explicit dimensions and add an editable, synchronized
  hex field that displays the current color, validates input, and resets with
  Use defaults.
- Show a centered spinner until the initial administration response arrives,
  preventing a premature read-only access message. Failed initial requests
  display an error with a Try again button.

## 0.5.0 — 2026-11-08

- Clarify the appearance changes on upgrade: the title defaults to "App Market"
  and the introduction banner and installation instruction are hidden by default.
  Configure your preferred title, header color, and Show subheader option in the
  Setup tab under Appearance, then save to apply your changes. Existing saved
  appearance settings are preserved.

## 0.4.10 — 2026-11-08

- Add a Show subheader checkbox under Setup → Appearance, disabled by default.
  It controls the Sandstorm introduction banner and installation instruction.
- This changes the default appearance of existing marketplaces on upgrade:
  both subheader messages are hidden. You may need to configure your preferred
  appearance in the Setup tab under Appearance, including the title, header
  color, and Show subheader option. Save appearance to apply your changes.

## 0.4.9 — 2026-11-08

- Add per-marketplace title and header color settings under Setup → Appearance.
  The title defaults to "App Market" and applies to page titles and headings.
- Derive header gradient and control colors from one base color, preserving the
  existing purple gradient at #552976 and choosing light or dark header text.
- Persist appearance settings across restarts and republish the site when saved.

## 0.4.8 — 2026-11-08

- Add icon for the installed app and marketplace listing.
- Use the modern "marketplace" noun phrase for creating new grains.
- Add project links, embedded release notes, public marketplace and administration
  screenshots, and visual asset attribution notices to package metadata.
- Set the maintainer contact to michael@nutt.im and support embedding a PGP
  author proof for the existing app ID.

## 0.4.7 — 2026-11-05

- Remember Sandstorm servers across marketplace visits, with a chooser to switch
  between saved servers or forget them. Navigation URLs no longer carry the host.
- Prefill private listing installation dialogs with the remembered server and
  save successful server selections.
- Automatically reject pending versions of the same app when approving a release
  if they were received earlier and have lower build numbers. Later submissions,
  existing review decisions, and releases without a known receipt order stay
  unchanged. Approval and automatic rejections are saved together with audit
  history and rejection reasons identifying the approved release.

## 0.4.6 — 2026-10-27

- Made the public header search focus outline surround both the input and button.
- Kept category focus outlines visible inside the scrolling header navigation.
- Adjusted the header logo’s vertical alignment.

## 0.4.5 — 2026-10-27

- Added a Homepage tab to pin, reorder, and hide published apps.
- Restored the public homepage’s legacy ordering and added a larger first-app
  card with a description excerpt.
- Sped up administration loads by skipping screenshot processing when reading
  review data. Optimized PGP verification arithmetic in development builds.
- Fixed SPK uploads and catalog imports failing on cleartext OpenPGP author
  proofs, including proofs with trailing armor whitespace. Signature, key
  binding, and app ownership checks remain enforced.
- Restored the legacy short-description overrides for Let’s Chat, MediaGoblin,
  FileDrop, and Paperwork during package validation and listing generation,
  without changing signed package metadata.

## 0.4.4 — 2026-10-27

- Added author pages that group apps by Keybase identity or PGP fingerprint, with
  profile pictures, social links, and links from app listings.
- Show names from signing PGP keys when no author profile is available, marked
  as unverified. Signature-only author pages explain how to connect the key to
  Keybase.
- Improved author presentation with larger names and social icons on app pages,
  and service icons beside usernames on author pages.
- Added a Debugging section under Setup with controls to rebuild the public site
  and refresh all authors’ Keybase profiles. Bulk refresh preserves existing
  profiles when lookups fail, reports failures, and rebuilds the site after
  successful updates.
- Cache downloaded SPKs during catalog syncs, reuse completed downloads on
  subsequent runs, and build import archives from the cache. Interrupted downloads
  are retried automatically.
- Added support for BZip2-compressed PGP author proofs, including Desert Atlas.

## 0.4.3 — 2026-10-22

- Fixed SPK imports failing when an author’s PGP key includes third-party
  certifications, including the affected MediaWiki package. Self-certifications,
  subkey bindings, and app ownership proofs remain cryptographically verified.

## 0.4.2 — 2026-10-21

- Added direct import of one or more SPK packages from the administration UI,
  including into existing marketplaces. Packages are verified and submitted for
  publication; new releases require approval and existing review decisions are
  preserved. Imports show per-file results and continue if a file fails.
- Added activity history and submission notifications for imported packages that
  change the publication request.
- Added a Web publishing section with a public preview link and custom-domain DNS
  instructions, including guidance for root domains.
- Renamed the Settings tab to Setup. Existing Settings and Notifications URLs
  continue to work.

## 0.4.1 — 2026-10-18

- Added a Settings page containing Keybase connection and submission notification
  settings. Existing Notifications URLs continue to work.
- Restored Connect Keybase, with saved connection status and a reconnect option.
- Made package IDs, receipt dates, and verified PGP fingerprints visible directly
  in version history. Removed the expandable package details section in favor of
  Preview listing.
- Added inline Keybase author lookups beside verified fingerprints when connected,
  showing author details and a profile image link in place of the lookup control.

## 0.4.0 — 2026-10-18

- Added configurable email notifications for app submissions.
- Added changelogs to public app listings and (restricted) Markdown rendering for
  descriptions and changelogs.
- Added configurable signing-key replacements and revocations, with package
  revalidation and audit history.
- Split administration into Apps, Notifications, and Key replacements tabs, with
  a pending-review badge and a link to the public site.
- Improved concurrent administration and submissions by queueing registry
  operations and moving upload and import preparation outside the write lock.
- Sped up public-site rebuilds by reusing unchanged images and batching disk
  writes, and accelerated SPK packaging with parallel compression.
- Replaced raw gateway errors in the admin interface with readable messages.
- Updated Rust dependencies, browser test tooling, and GitHub Actions.

## 0.3.0 — 2026-10-17

- Added private, paginated activity history for each app, covering submissions,
  review decisions, publication, withdrawals, rollbacks, and deprecation. Events
  include actor attribution and detailed records.
- **BREAKING CHANGE**: Centralized registry metadata and audit history in SQLite,
  with transactional state changes and recovery of interrupted publication updates.
  SPK bundles remain separate files.
- Added persistent admin app URLs, supporting direct links, reloads, and browser
  back/forward navigation within Sandstorm.
- Refined the administration interface, including app information, review actions,
  approval queue navigation, and links to published app pages.
- Added release build cache warming on pushes to `main` to speed up tagged builds.

## 0.2.0 — 2026-10-16

- Added app deprecation with optional replacement links. Deprecated apps are hidden
  from discovery while their direct pages and package downloads remain available.
  Administrators can restore them to the indexes at any time.

## 0.1.0

- Initial app release
