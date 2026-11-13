<header class="page-title">
    <div class="wrap">
        <?php if (!empty($eyebrow)): ?>
        <p class="eyebrow"><?= escape($eyebrow) ?></p>
        <?php endif; ?>
        <h1><?= escape($heading) ?></h1>
        <?php if (!empty($subtitle)): ?>
        <p class="lede"><?= escape($subtitle) ?></p>
        <?php endif; ?>
    </div>
</header>
