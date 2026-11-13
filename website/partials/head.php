<?php
$type = $type ?? 'website';
$published = $published ?? null;
$robots = $robots ?? null;
$structuredData = $structuredData ?? [];
$previewImage = SITE_URL . '/link-preview.png';
$previewAlt = 'The Voximov timeline with a video, a recorded voice, an AI script clip and music';
?>
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title><?= escape($title) ?></title>
    <meta name="description" content="<?= escape($description) ?>">
    <?php if ($robots): ?>
    <meta name="robots" content="<?= escape($robots) ?>">
    <?php endif; ?>
    <link rel="canonical" href="<?= escape(canonical_url()) ?>">
    <link rel="alternate" type="application/rss+xml" title="Voximov Blog" href="/feed.xml">
    <link rel="describedby" href="/llms.txt" type="text/markdown">
    <link rel="icon" href="/favicon.svg" type="image/svg+xml">
    <link rel="icon" href="/favicon-32x32.png" type="image/png" sizes="32x32">
    <link rel="apple-touch-icon" href="/apple-touch-icon.png" sizes="180x180">
    <meta name="theme-color" content="#111315">
    <meta property="og:site_name" content="Voximov">
    <meta property="og:locale" content="en_US">
    <meta property="og:type" content="<?= escape($type) ?>">
    <meta property="og:title" content="<?= escape($title) ?>">
    <meta property="og:description" content="<?= escape($description) ?>">
    <meta property="og:url" content="<?= escape(canonical_url()) ?>">
    <meta property="og:image" content="<?= escape($previewImage) ?>">
    <meta property="og:image:width" content="1200">
    <meta property="og:image:height" content="630">
    <meta property="og:image:type" content="image/png">
    <meta property="og:image:alt" content="<?= escape($previewAlt) ?>">
    <?php if ($type === 'article' && $published): ?>
    <meta property="article:published_time" content="<?= escape($published) ?>">
    <?php endif; ?>
    <meta name="twitter:card" content="summary_large_image">
    <meta name="twitter:title" content="<?= escape($title) ?>">
    <meta name="twitter:description" content="<?= escape($description) ?>">
    <meta name="twitter:image" content="<?= escape($previewImage) ?>">
    <meta name="twitter:image:alt" content="<?= escape($previewAlt) ?>">
    <?php foreach ($structuredData as $item): ?>
    <script type="application/ld+json"><?= json_encode($item, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE | JSON_HEX_TAG) ?></script>
    <?php endforeach; ?>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Golos+Text:wght@400;500;600;700&display=swap">
    <link rel="stylesheet" href="<?= escape(asset('/styles.css')) ?>">
</head>
<body>
