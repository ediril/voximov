<?php

function check(bool $condition, string $message): void {
    if (!$condition) {
        fwrite(STDERR, "Failed: $message\n");
        exit(1);
    }
}

function render(string $path): array {
    $root = dirname(__DIR__);
    $script = sprintf(
        '$_SERVER["REQUEST_URI"] = %s; chdir(%s); require "router.php";',
        var_export($path, true),
        var_export($root, true)
    );
    $output = [];
    exec('php -d display_errors=stderr -r ' . escapeshellarg($script) . ' 2>&1', $output, $status);
    return [$status, implode("\n", $output)];
}

[, $home] = render('/?ref=test');
check(str_contains($home, '<link rel="canonical" href="https://voximov.com/">'), 'The home page canonical URL drops query strings');
check(str_contains($home, '"@type":"SoftwareApplication"'), 'The home page describes the app for search engines');
check(str_contains($home, 'name="access_key"') && str_contains($home, 'type="email"'), 'The home page has the email sign-up form');
check(!str_contains($home, 'Warning:') && !str_contains($home, 'Fatal error'), 'The home page renders without PHP errors');

foreach (['/pricing', '/privacy', '/terms'] as $linked) {
    [, $page] = render($linked);
    check(str_contains($home, 'href="' . $linked . '"'), "Every page links to $linked");
    check(str_contains($page, '<link rel="canonical" href="https://voximov.com' . $linked . '">') && !str_contains($page, 'noindex'), "$linked is a real page");
}

[, $blog] = render('/blog');
check(str_contains($blog, 'href="/blog/add-a-voiceover-to-a-screen-recording-on-mac"'), 'The blog lists published posts');

[, $post] = render('/blog/add-a-voiceover-to-a-screen-recording-on-mac');
check(str_contains($post, '<meta property="og:type" content="article">'), 'Posts are Open Graph articles');
check(str_contains($post, '"@type":"BlogPosting"'), 'Posts carry BlogPosting structured data');
check(str_contains($post, '<link rel="canonical" href="https://voximov.com/blog/add-a-voiceover-to-a-screen-recording-on-mac">'), 'Posts have their own canonical URL');

[, $missing] = render('/blog/no-such-post');
check(str_contains($missing, '<meta name="robots" content="noindex, follow">'), 'Unknown posts get the 404 page');

[, $hidden] = render('/_blog_posts/add-a-voiceover-to-a-screen-recording-on-mac.md');
check(!str_contains($hidden, 'title: How to'), 'Post sources are not served');

[, $sitemap] = render('/sitemap.xml');
check(str_contains($sitemap, '<loc>https://voximov.com/blog/add-a-voiceover-to-a-screen-recording-on-mac</loc>'), 'The sitemap lists posts');
check(simplexml_load_string(substr($sitemap, strpos($sitemap, '<?xml'))) !== false, 'The sitemap is valid XML');

[, $feed] = render('/feed.xml');
check(simplexml_load_string(substr($feed, strpos($feed, '<?xml'))) !== false, 'The feed is valid XML');

echo "Page tests passed\n";
