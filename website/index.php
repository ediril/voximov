<?php
require_once __DIR__ . '/common.php';

$features = [
    ['tone' => 'lime', 'label' => 'Record', 'title' => 'Record at the playhead', 'text' => 'Put on headphones, press R and talk while the video and music play. Each take lands on the armed track, right where you started.'],
    ['tone' => 'blue', 'label' => 'Edit', 'title' => 'Cut every track at once', 'text' => 'Press S to split every track at the playhead, on exact video frames. Drag clips between tracks, copy and paste sections, and undo any edit.'],
    ['tone' => 'gold', 'label' => 'Sound', 'title' => 'Sound finished', 'text' => 'Voice presets with high-pass, equalizer, compressor and expander. Loudness is normalized to −16 LUFS with a −1.5 dBTP ceiling, so every video comes out at the same level.'],
    ['tone' => 'rose', 'label' => 'Mix', 'title' => 'Balance the tracks', 'text' => 'Volume, pan, mute and solo on every track. Mute the video track to leave its own sound out of the export.'],
    ['tone' => 'lime', 'label' => 'Export', 'title' => 'Export the finished MP4', 'text' => 'Export the edited video with the full mix, encoded on your Mac’s hardware H.264 encoder. Or export the mix or a single track as WAV or MP3.'],
    ['tone' => 'blue', 'label' => 'Project', 'title' => 'Originals stay untouched', 'text' => 'Your video, voice and music files never change. One .voximov project file holds the sources, takes and edits, and every save is crash-safe.'],
];

$scriptRows = [
    ['time' => '0:01.2', 'text' => 'Open the menu and import your video.', 'status' => 'Ready', 'tone' => 'fits'],
    ['time' => '0:08.6', 'text' => 'Press R to record while it plays.', 'status' => 'Ready', 'tone' => 'fits'],
    ['time' => '0:16.4', 'text' => 'Then send the MP4 to Vidmuse.', 'status' => 'Too long', 'tone' => 'overflow', 'guessed' => 'Vidmuse'],
];

$languages = ['English', 'Deutsch', 'Español', 'Français', 'Italiano', 'Polski', 'Português', 'Русский', 'Türkçe', 'Українська', '日本語', '한국어', '简体中文'];

$structuredData = [
    [
        '@context' => 'https://schema.org',
        '@type' => 'WebSite',
        'name' => 'Voximov',
        'url' => SITE_URL . '/',
        'description' => SITE_PITCH,
    ],
    [
        '@context' => 'https://schema.org',
        '@type' => 'SoftwareApplication',
        'name' => 'Voximov',
        'operatingSystem' => 'macOS',
        'applicationCategory' => 'MultimediaApplication',
        'url' => SITE_URL . '/',
        'description' => SITE_PITCH,
    ],
];

echo partial('head', [
    'title' => 'Voximov: Record and Edit Voiceovers Over Your Video on Mac',
    'description' => SITE_PITCH,
    'structuredData' => $structuredData,
]);
echo partial('nav');
?>
<main>
    <header class="hero">
        <div class="wrap">
            <p class="eyebrow">Voiceover studio for Mac</p>
            <h1><?= escape(SITE_HEADLINE) ?></h1>
            <p class="lede"><?= escape(SITE_PITCH) ?></p>
            <div class="hero-actions">
                <a href="#early-access" class="button">Get early access</a>
                <a href="#features" class="button button-quiet">See how it works</a>
            </div>
            <?= partial('studio') ?>
        </div>
    </header>

    <section class="section problem">
        <div class="wrap narrow center">
            <h2>Stop juggling apps to add a voice</h2>
            <p class="section-lede">Record in one app, line it up in another, and hope the levels match. Voximov puts your video, your voice and your music on one timeline, so you record, cut, mix and export in the same window.</p>
        </div>
    </section>

    <section class="section" id="features">
        <div class="wrap">
            <div class="section-head">
                <p class="eyebrow">Features</p>
                <h2>Everything a voiceover needs, nothing it doesn’t</h2>
            </div>
            <div class="cards">
                <?php foreach ($features as $feature): ?>
                <article class="card card-<?= $feature['tone'] ?>">
                    <p class="card-label"><?= escape($feature['label']) ?></p>
                    <h3><?= escape($feature['title']) ?></h3>
                    <p><?= escape($feature['text']) ?></p>
                </article>
                <?php endforeach; ?>
            </div>
        </div>
    </section>

    <section class="section section-alt" id="ai-voiceover">
        <div class="wrap split">
            <div>
                <p class="eyebrow">AI voiceover</p>
                <h2>Rather not talk? Type the script.</h2>
                <p class="section-lede">No retakes for every stumble, and no listening to your own voice. Choose the Script tool and drag across an audio track. The clip’s width is the time the line may take. Type the line, press Enter, and an AI voice speaks it in place. If the speech runs longer than the clip, you see exactly how much.</p>
                <ul class="checks">
                    <li>28 American and British voices, with a speed setting for each track.</li>
                    <li>Knows most English words and common tech terms such as Kubernetes, npm and JSON.</li>
                    <li>For any other word it makes a guess and lists the words it guessed, so you can listen and check them.</li>
                    <li>For an exact pronunciation, write the sounds in the script: <code>[Voximov](/vˈɑksɪmɑv/)</code></li>
                </ul>
            </div>
            <div class="script-panel" role="img" aria-label="The Voximov script list with three generated lines, one too long for its clip, and the word Vidmuse marked as a guessed pronunciation">
                <div class="script-panel-head">
                    <span>Script clips</span>
                    <span class="script-voice">Heart (US female)</span>
                </div>
                <?php foreach ($scriptRows as $row): ?>
                <div class="script-row">
                    <span class="script-time"><?= escape($row['time']) ?></span>
                    <span class="script-text"><?= escape($row['text']) ?></span>
                    <span class="script-chip is-<?= $row['tone'] ?>"><?= escape($row['status']) ?></span>
                    <?php if (!empty($row['guessed'])): ?>
                    <span class="script-guessed">Guessed pronunciation: <?= escape($row['guessed']) ?></span>
                    <?php endif; ?>
                </div>
                <?php endforeach; ?>
            </div>
        </div>
    </section>

    <section class="section" id="private">
        <div class="wrap">
            <div class="section-head">
                <p class="eyebrow">Private by design</p>
                <h2>Runs on your Mac, not in someone’s cloud</h2>
            </div>
            <div class="facts">
                <div class="fact">
                    <h3>No account, no upload</h3>
                    <p>Your video, your voice and your script never leave your computer. There is nothing to sign up for.</p>
                </div>
                <div class="fact">
                    <h3>The AI voice is local too</h3>
                    <p>Voices come from Kokoro, an open-source voice model. Voximov downloads it once, about 100 MB, the first time you generate speech. After that it works offline.</p>
                </div>
                <div class="fact">
                    <h3>Speaks your language</h3>
                    <p>The interface comes in 13 languages: <?= implode(', ', array_map(static fn(string $language): string => '<span class="nowrap">' . escape($language) . '</span>', $languages)) ?>.</p>
                </div>
            </div>
        </div>
    </section>

    <section class="section section-alt">
        <div class="wrap narrow center">
            <p class="eyebrow">Who it’s for</p>
            <h2>For people who narrate their own videos</h2>
            <p class="section-lede">Tutorials, game footage, product demos, course lessons: anything that needs a voice on top of the picture. Voximov works best for one narrator and a finished video. It is not a full video editor, and it never changes your original video.</p>
            <ul class="tags">
                <li>Tutorials</li>
                <li>Game footage</li>
                <li>Product demos</li>
                <li>Course lessons</li>
                <li>Screen recordings</li>
            </ul>
        </div>
    </section>

    <?= partial('waitlist') ?>
</main>
<?= partial('footer') ?>
