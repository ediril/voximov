<?php
require_once __DIR__ . '/common.php';

header('Content-Type: application/xml; charset=UTF-8');

$changed = static function (string $page): string {
    $files = array_merge([__DIR__ . '/' . $page, __DIR__ . '/common.php', __DIR__ . '/styles.css'], glob(__DIR__ . '/partials/*.php'));
    return gmdate('Y-m-d', max(array_map('filemtime', $files)));
};

$posts = get_all_blog_posts();
$urls = [
    ['loc' => SITE_URL . '/', 'lastmod' => $changed('index.php')],
    ['loc' => SITE_URL . '/blog', 'lastmod' => max([$changed('blog.php'), ...array_column($posts, 'date')])],
    ['loc' => SITE_URL . '/pricing', 'lastmod' => $changed('pricing.php')],
    ['loc' => SITE_URL . '/privacy', 'lastmod' => $changed('privacy.php')],
    ['loc' => SITE_URL . '/terms', 'lastmod' => $changed('terms.php')],
];
foreach ($posts as $post) {
    $urls[] = ['loc' => SITE_URL . '/blog/' . $post['slug'], 'lastmod' => $post['date']];
}

echo '<?xml version="1.0" encoding="UTF-8"?>' . "\n";
?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
<?php foreach ($urls as $url): ?>
  <url>
    <loc><?= htmlspecialchars($url['loc'], ENT_XML1 | ENT_QUOTES, 'UTF-8') ?></loc>
    <lastmod><?= htmlspecialchars($url['lastmod'], ENT_XML1 | ENT_QUOTES, 'UTF-8') ?></lastmod>
  </url>
<?php endforeach; ?>
</urlset>
