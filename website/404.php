<?php
http_response_code(404);
require_once __DIR__ . '/common.php';

echo partial('head', [
    'title' => 'Page not found - Voximov',
    'description' => 'The requested Voximov page was not found.',
    'robots' => 'noindex, follow',
]);
echo partial('nav');
?>
<main>
    <?= partial('page-title', ['eyebrow' => '404', 'heading' => 'Page not found', 'subtitle' => 'The page may have moved, or it may no longer exist.']) ?>
    <section class="wrap prose-page">
        <a href="/" class="button">Back to Voximov</a>
    </section>
</main>
<?= partial('footer') ?>
