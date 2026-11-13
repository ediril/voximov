<?php
require_once __DIR__ . '/common.php';

echo partial('head', [
    'title' => 'Privacy - Voximov',
    'description' => 'What Voximov and voximov.com collect: the app keeps your work on your Mac, and the website collects only the email you choose to leave.',
]);
echo partial('nav');
?>
<main>
    <?= partial('page-title', ['eyebrow' => 'Privacy', 'heading' => 'Your work stays on your Mac', 'subtitle' => 'The short version: the app collects nothing, and this website collects only the email address you choose to leave.']) ?>
    <article class="wrap prose-page">
        <div class="prose">
            <h2>The Voximov app</h2>
            <ul>
                <li>Voximov runs on your computer. Your videos, recordings, scripts and projects stay there, and the app never uploads them.</li>
                <li>There is no account, no sign-in, no analytics and no usage tracking in the app.</li>
                <li>The first time you generate AI speech, Voximov downloads the Kokoro voice model, about 100 MB, from Hugging Face. Hugging Face receives that download request the same way any website receives a visit. Your script is not sent; speech is generated on your computer.</li>
                <li>Recording asks for microphone access, which macOS lets you grant or revoke at any time.</li>
            </ul>
            <h2>This website</h2>
            <ul>
                <li>If you join the early-access list, your email address is sent to us through our form provider, <a href="https://web3forms.com/">Web3Forms</a>. We use it only to tell you about Voximov, and we never sell or share it.</li>
                <li>We count visits with <a href="https://www.simpleanalytics.com/">Simple Analytics</a>, which uses no cookies and collects no personal data.</li>
                <li>The site sets no cookies of its own.</li>
            </ul>
            <h2>Your choices</h2>
            <p>To have your email address removed from the list, or for any question about privacy, write to <a href="mailto:<?= escape(CONTACT_EMAIL) ?>"><?= escape(CONTACT_EMAIL) ?></a>.</p>
        </div>
    </article>
</main>
<?= partial('footer') ?>
