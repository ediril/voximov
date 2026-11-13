<?php
require_once __DIR__ . '/common.php';

$licenseUrl = 'https://polyformproject.org/licenses/strict/1.0.0/';
$commercialMail = 'mailto:' . CONTACT_EMAIL . '?subject=' . rawurlencode('Voximov commercial license');
$questionMail = 'mailto:' . CONTACT_EMAIL . '?subject=' . rawurlencode('Voximov licensing question');

echo partial('head', [
    'title' => 'Pricing - Voximov',
    'description' => 'Voximov is free for noncommercial and personal use and for nonprofit, educational and government organizations. Commercial use needs a paid license.',
]);
echo partial('nav');
?>
<main>
    <?= partial('page-title', ['eyebrow' => 'Pricing', 'heading' => 'Free for personal use. Licensed for business.', 'subtitle' => 'Voximov is free for noncommercial and personal use, and for nonprofit, educational and government organizations. Commercial use needs a paid license.', 'centered' => true]) ?>
    <section class="wrap plans-section">
        <div class="plans">
            <article class="plan" aria-labelledby="free-plan">
                <p class="plan-label">PolyForm Strict 1.0.0</p>
                <h2 id="free-plan">Free</h2>
                <p class="plan-intro">No charge for the uses the license permits:</p>
                <ul class="checks">
                    <li>Any noncommercial purpose.</li>
                    <li>Personal study, private entertainment and hobby projects, with no anticipated commercial application.</li>
                    <li>Charitable organizations, educational institutions, public research organizations, public safety or health organizations, environmental protection organizations and government institutions.</li>
                </ul>
                <a class="button button-quiet" href="<?= escape($licenseUrl) ?>">Read the license</a>
            </article>
            <article class="plan plan-featured" aria-labelledby="commercial-plan">
                <p class="plan-label">Commercial</p>
                <h2 id="commercial-plan">Contact for pricing</h2>
                <p class="plan-intro">For commercial use outside the free license, such as:</p>
                <ul class="checks">
                    <li>Videos made for clients or as paid work.</li>
                    <li>Product demos, tutorials and marketing videos for a business.</li>
                    <li>Courses and other videos you sell or earn money from.</li>
                </ul>
                <a class="button" href="<?= escape($commercialMail) ?>">Request a commercial license</a>
                <p class="plan-note">A separate paid agreement is required before commercial use outside the free license.</p>
            </article>
        </div>
        <p class="plans-note">The license does not allow modifying, redistributing or reselling Voximov. The <a href="<?= escape($licenseUrl) ?>">full license</a> governs; this summary grants no additional permissions. Not sure which applies to you? <a href="<?= escape($questionMail) ?>">Ask us</a>.</p>
    </section>
    <?= partial('waitlist') ?>
</main>
<?= partial('footer') ?>
