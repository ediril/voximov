<section class="waitlist" id="early-access">
    <div class="wrap waitlist-inner">
        <h2><?= escape($heading ?? 'Be the first to try Voximov') ?></h2>
        <p><?= escape($text ?? 'Voximov is in early access for Macs with Apple Silicon. Leave your email and we will let you know when it is ready to download.') ?></p>
        <form class="waitlist-form" action="https://api.web3forms.com/submit" method="POST">
            <input type="hidden" name="access_key" value="<?= escape(WEB3FORMS_ACCESS_KEY) ?>">
            <input type="hidden" name="subject" value="Voximov early access signup">
            <input type="hidden" name="from_name" value="voximov.com">
            <input type="checkbox" name="botcheck" hidden tabindex="-1" autocomplete="off">
            <div class="waitlist-row">
                <input type="email" name="email" class="waitlist-input" placeholder="you@example.com" aria-label="Email address" autocomplete="email" required>
                <button type="submit" class="button">Get early access</button>
            </div>
            <p class="waitlist-status" role="status" hidden></p>
        </form>
        <p class="waitlist-note">Your email is used only to tell you about Voximov.</p>
    </div>
</section>
<script src="<?= escape(asset('/waitlist.js')) ?>" defer></script>
