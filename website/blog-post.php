<?php
require_once __DIR__ . '/common.php';

$post = get_blog_post_by_slug((string)($_GET['slug'] ?? ''));
if (!$post) {
    require __DIR__ . '/404.php';
    exit;
}

$description = $post['excerpt'] ?: 'A post from the Voximov blog.';
$article = [
    '@context' => 'https://schema.org',
    '@type' => 'BlogPosting',
    'headline' => $post['title'],
    'description' => $description,
    'datePublished' => $post['date'],
    'url' => SITE_URL . '/blog/' . $post['slug'],
    'image' => SITE_URL . '/link-preview.png',
    'author' => ['@type' => 'Organization', 'name' => 'Voximov', 'url' => SITE_URL . '/'],
    'publisher' => ['@type' => 'Organization', 'name' => 'Voximov', 'url' => SITE_URL . '/'],
];

echo partial('head', [
    'title' => $post['title'] . ' - Voximov Blog',
    'description' => $description,
    'type' => 'article',
    'published' => $post['date'] . 'T00:00:00Z',
    'structuredData' => [$article],
]);
echo partial('nav');
?>
<main>
    <header class="page-title">
        <div class="wrap">
            <a href="/blog" class="back-link">&larr; All posts</a>
            <h1><?= escape($post['title']) ?></h1>
            <time class="post-meta" datetime="<?= escape($post['date']) ?>"><?= escape(format_blog_date($post['date'])) ?></time>
        </div>
    </header>
    <article class="wrap prose-page">
        <div class="prose">
            <?= blog_markdown_to_html($post['content']) ?>
        </div>
    </article>
    <?= partial('waitlist', ['heading' => 'Try Voximov early', 'text' => 'Voximov records, edits and mixes your voiceover over the video, on your Mac. Leave your email to hear when it is ready to download.']) ?>
</main>
<?= partial('footer') ?>
