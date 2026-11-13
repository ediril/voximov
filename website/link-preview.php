<?php
require_once __DIR__ . '/common.php';
?>
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="robots" content="noindex">
    <title>Voximov link preview</title>
    <link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Golos+Text:wght@400;500;600;700&display=swap">
    <link rel="stylesheet" href="<?= escape(asset('/styles.css')) ?>">
    <style>
        body { width: 1200px; height: 630px; overflow: hidden; padding: 56px 64px 0; }
        .preview-head { display: flex; align-items: flex-start; justify-content: space-between; gap: 40px; margin-bottom: 40px; }
        .preview-head h1 { font-size: 62px; font-weight: 700; letter-spacing: -0.035em; max-width: 720px; }
        .preview-head .brand { font-size: 26px; margin-top: 10px; }
        .studio { width: 1072px; }
        .studio-playhead span { animation: none; }
    </style>
</head>
<body>
    <div class="preview-head">
        <h1><?= escape(SITE_HEADLINE) ?></h1>
        <span class="brand"><img src="/favicon.svg" alt="" width="40" height="40"><span>Voximov</span></span>
    </div>
    <?= partial('studio') ?>
</body>
</html>
