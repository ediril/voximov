<?php
require_once __DIR__ . '/common.php';

header('Content-Type: application/rss+xml; charset=UTF-8');

$xml = static fn(string $text): string => htmlspecialchars($text, ENT_XML1 | ENT_QUOTES, 'UTF-8');
$posts = get_all_blog_posts();

echo '<?xml version="1.0" encoding="UTF-8"?>' . "\n";
?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
<channel>
  <title>Voximov Blog</title>
  <link><?= $xml(SITE_URL . '/blog') ?></link>
  <atom:link href="<?= $xml(SITE_URL . '/feed.xml') ?>" rel="self" type="application/rss+xml"/>
  <description>Guides on recording voiceovers, narrating screen recordings and using AI voices on a Mac.</description>
  <language>en</language>
<?php foreach ($posts as $post): ?>
  <item>
    <title><?= $xml($post['title']) ?></title>
    <link><?= $xml(SITE_URL . '/blog/' . $post['slug']) ?></link>
    <guid><?= $xml(SITE_URL . '/blog/' . $post['slug']) ?></guid>
    <pubDate><?= gmdate(DATE_RSS, strtotime($post['date'] . 'T00:00:00Z')) ?></pubDate>
    <description><?= $xml($post['excerpt']) ?></description>
  </item>
<?php endforeach; ?>
</channel>
</rss>
