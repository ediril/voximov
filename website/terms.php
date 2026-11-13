<?php
require_once __DIR__ . '/common.php';

echo partial('head', [
    'title' => 'Terms - Voximov',
    'description' => 'The terms for using the Voximov app and voximov.com: who may use Voximov free of charge, commercial licenses, your content, and limits of liability.',
]);
echo partial('nav');
?>
<main>
    <?= partial('page-title', ['eyebrow' => 'Terms', 'heading' => 'Terms of use', 'subtitle' => 'Last updated November 13, 2026.']) ?>
    <article class="wrap prose-page">
        <div class="prose">
            <h2>Using the Voximov app</h2>
            <p>Voximov is licensed, not sold. Your use of the app is governed by the <a href="https://polyformproject.org/licenses/strict/1.0.0/">PolyForm Strict License 1.0.0</a>. Under that license, Voximov is free of charge for:</p>
            <ul>
                <li>Any noncommercial purpose.</li>
                <li>Personal use such as personal study, private entertainment and hobby projects, with no anticipated commercial application.</li>
                <li>Charitable organizations, educational institutions, public research organizations, public safety or health organizations, environmental protection organizations and government institutions.</li>
            </ul>
            <p>Any other commercial use requires a separate paid license, agreed before you start such use. See <a href="/pricing">pricing</a>.</p>
            <p>The license does not allow you to modify, redistribute or resell Voximov, or to build other works from it. If anything on this page differs from the license, the license applies.</p>

            <h2>Your content</h2>
            <p>Your videos, recordings, scripts, projects and the audio and video you export are yours. Voximov claims no rights to them. You are responsible for having the rights to the media you import and for how you use what you make, including AI-generated speech. Do not use AI voices to impersonate a real person or to mislead people about who is speaking.</p>

            <h2>Early access</h2>
            <p>Voximov is pre-release software. Features may change, and it may contain bugs. Keep copies of your original media and back up your projects.</p>

            <h2>No warranty</h2>
            <p>As far as the law allows, Voximov and this website are provided as is, without any warranty or condition, and we are not liable to you for any damages arising from them.</p>

            <h2>This website</h2>
            <p>The <a href="/privacy">privacy page</a> describes what this website collects, including the email address you leave to join the early-access list.</p>

            <h2>Changes and contact</h2>
            <p>We may update these terms. The date at the top of this page shows when they last changed. For any question, write to <a href="mailto:<?= escape(CONTACT_EMAIL) ?>"><?= escape(CONTACT_EMAIL) ?></a>.</p>
        </div>
    </article>
</main>
<?= partial('footer') ?>
