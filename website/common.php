<?php

require_once __DIR__ . '/blog_content.php';

const SITE_URL = 'https://voximov.com';
const SITE_HEADLINE = 'Narrate your videos in one window.';
const SITE_PITCH = 'Record, edit and mix a voiceover right on top of your video, or type a script and let an AI voice read it. All on your Mac, with nothing uploaded.';
const CONTACT_EMAIL = 'emrah@diril.org';
const WEB3FORMS_ACCESS_KEY = '2235f0eb-2c33-4ae3-8b72-77ed6f365707';

function escape(string $text): string {
    return htmlspecialchars($text, ENT_QUOTES | ENT_SUBSTITUTE, 'UTF-8');
}

function partial(string $name, array $variables = []): string {
    extract($variables);
    ob_start();
    include __DIR__ . '/partials/' . $name . '.php';
    return ob_get_clean();
}

function request_path(): string {
    return parse_url($_SERVER['REQUEST_URI'] ?? '/', PHP_URL_PATH) ?: '/';
}

function canonical_url(): string {
    $path = request_path();
    return SITE_URL . ($path === '/' ? '/' : rtrim($path, '/'));
}

function asset(string $path): string {
    return $path . '?v=' . filemtime(__DIR__ . $path);
}

function waveform(int $seed, int $bars = 96, float $loudness = 1.0, bool $speech = true): string {
    $rects = '';
    for ($bar = 0; $bar < $bars; $bar++) {
        $phrase = abs(sin(($bar + $seed * 7) * 0.11)) * 0.7 + 0.3;
        if ($speech && sin($bar * 0.37 + $seed) < -0.55) {
            $phrase = 0.04;
        }
        $grain = abs(sin(($bar + 1) * 12.9898 + $seed * 78.233) * 43758.5453);
        $grain -= floor($grain);
        $height = max(2, round(36 * $loudness * $phrase * (0.35 + 0.65 * $grain), 1));
        $rects .= sprintf('<rect x="%d" y="%.1f" width="2" height="%.1f" rx="1"/>', $bar * 3, (40 - $height) / 2, $height);
    }
    return sprintf('<svg class="wave" viewBox="0 0 %d 40" preserveAspectRatio="none" aria-hidden="true">%s</svg>', $bars * 3, $rects);
}
