# Website and blog

Date: 2026-11-13

## Decision

The voximov.com website and blog live in `website/` as plain PHP with no
build step.

- Serve it locally with `scripts/run_server.sh [port]` (default 8000). Test it
  with `npm run test:website`.
- A blog post is a Markdown file in `website/_blog_posts/`. Its file name is
  the URL slug, and its front matter needs `title`, `date` and `excerpt`.
- Website text is English only and does not go in `src/locales`.
- Every product claim on the site must match the README. A feature is claimed
  only once it exists in the app.
- After changing the headline or the hero illustration, re-render
  `website/link-preview.png` from `/link-preview.php` at 1200×630.
- The early-access form posts to Web3Forms, which emails the owner. The site
  uses no analytics and no cookies.

## Rationale

The site is a landing page, a pricing page, legal pages and a blog for search.
Plain PHP matches how the owner's other sites are built and deployed (nginx
and php-fpm), and needs no build pipeline. Matching the README keeps the site
from promising features the app does not have.

## Boundaries

- Production is served by nginx with `website/nginx.conf`.
- Switching to a real mailing list provider, or adding analytics, is a new
  decision. The privacy page must change with it.
