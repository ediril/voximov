# Every app string lives in src/locales

Date: 2026-10-05

## Decision

All user-facing text in the app lives in `src/locales`. English is the
complete fallback catalog. When a string is added or changed, every supported
language is updated in the same change, and interpolation parameters are kept
exactly as in English.

Documentation is written in English only. The website is English only and
does not use `src/locales` (see
[the website note](2026-11-13-website-and-blog.md)).

## Rationale

The interface ships in 13 languages. A string added only in English shows up
untranslated in the other twelve, and a dropped interpolation parameter shows
a broken message. Updating every language in the same change keeps the
catalogs complete.

## Mechanism

`tests/localization.test.ts` checks the catalogs against the English one.
