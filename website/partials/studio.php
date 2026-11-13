<?php
$scenes = [212, 218, 200, 196, 226, 232, 205, 190, 214, 221, 208, 199];
$lanes = [
    ['name' => 'Video', 'kind' => 'video'],
    ['name' => 'Voice', 'kind' => 'voice', 'clips' => [[3, 27, 3, 1.0], [34, 21, 5, 0.9]]],
    ['name' => 'Script', 'kind' => 'script', 'clips' => [[59, 33, 11, 0.8]]],
    ['name' => 'Music', 'kind' => 'music', 'clips' => [[0, 100, 17, 0.45]]],
];
?>
<figure class="studio" role="img" aria-label="The Voximov timeline: a video track, a recorded voice split into two takes, an AI script clip that says “Then export it as an MP4.”, and a music track under the playhead">
    <div class="studio-bar">
        <span class="studio-dots"><i></i><i></i><i></i></span>
        <span class="studio-file">product-demo.voximov</span>
    </div>
    <div class="studio-body">
        <div class="studio-ruler">
            <span class="lane-name"></span>
            <span class="lane-area">
                <?php foreach (['0:00', '0:05', '0:10', '0:15', '0:20', '0:25'] as $index => $label): ?>
                <span class="tick" style="left: <?= $index * 18 ?>%"><?= $label ?></span>
                <?php endforeach; ?>
            </span>
        </div>
        <?php foreach ($lanes as $lane): ?>
        <div class="lane lane-<?= $lane['kind'] ?>">
            <span class="lane-name"><?= escape($lane['name']) ?></span>
            <span class="lane-area">
                <?php if ($lane['kind'] === 'video'): ?>
                <span class="clip clip-video" style="left: 0; width: 100%">
                    <?php foreach ($scenes as $index => $hue): ?>
                    <i style="--hue: <?= $hue ?>; --light: <?= 26 + ($index * 7) % 14 ?>%"></i>
                    <?php endforeach; ?>
                </span>
                <?php else: ?>
                <?php foreach ($lane['clips'] as [$left, $width, $seed, $loudness]): ?>
                <span class="clip clip-<?= $lane['kind'] ?>" style="left: <?= $left ?>%; width: <?= $width ?>%">
                    <?php if ($lane['kind'] === 'script'): ?>
                    <span class="clip-text">Then export it as an MP4.</span>
                    <?php endif; ?>
                    <?= waveform($seed, (int)round($width * 2.4), $loudness, $lane['kind'] !== 'music') ?>
                </span>
                <?php endforeach; ?>
                <?php endif; ?>
            </span>
        </div>
        <?php endforeach; ?>
        <div class="studio-playhead"><span></span></div>
    </div>
</figure>
