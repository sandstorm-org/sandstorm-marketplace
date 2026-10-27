# Changelog

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
