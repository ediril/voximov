<?php
require_once __DIR__ . '/common.php';

$description = 'Guides on recording voiceovers, narrating screen recordings and using AI voices on a Mac, from the team behind Voximov.';
$posts = get_all_blog_posts();

echo partial('head', ['title' => 'Blog - Voximov', 'description' => $description]);
echo partial('nav');
?>
<main>
    <?= partial('page-title', ['eyebrow' => 'Blog', 'heading' => 'Notes on narrating videos', 'subtitle' => $description]) ?>
    <section class="wrap blog-list">
        <?php if (!$posts): ?>
        <p class="blog-empty">No posts yet.</p>
        <?php endif; ?>
        <?php foreach ($posts as $post): ?>
        <a class="post-link" href="/blog/<?= escape($post['slug']) ?>">
            <time datetime="<?= escape($post['date']) ?>"><?= escape(format_blog_date($post['date'])) ?></time>
            <h2><?= escape($post['title']) ?></h2>
            <?php if ($post['excerpt'] !== ''): ?>
            <p><?= escape($post['excerpt']) ?></p>
            <?php endif; ?>
        </a>
        <?php endforeach; ?>
    </section>
    <?= partial('waitlist') ?>
</main>
<?= partial('footer') ?>
