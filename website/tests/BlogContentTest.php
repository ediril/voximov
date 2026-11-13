<?php

require_once __DIR__ . '/../blog_content.php';

function check(bool $condition, string $message): void {
    if (!$condition) {
        fwrite(STDERR, "Failed: $message\n");
        exit(1);
    }
}

$fixtures = __DIR__ . '/fixtures/blog';

$posts = get_all_blog_posts($fixtures);
check(array_column($posts, 'slug') === ['record-in-short-takes', 'an-older-post'], 'Only well-named, dated posts are listed, newest first');
check($posts[1]['title'] === 'An older post', 'Quoted front-matter titles are unquoted');

$post = get_blog_post_by_slug('record-in-short-takes', $fixtures);
check($post !== null && $post['excerpt'] === 'Short takes are easier to fix.', 'A post resolves by its file name');
check(get_blog_post_by_slug('../common', $fixtures) === null, 'Path traversal does not resolve files');
check(get_blog_post_by_slug('Not_A_Slug', $fixtures) === null, 'Files without a URL-safe name are not served');

$html = blog_markdown_to_html($post['content']);
check(str_contains($html, '<h2>Start with the notes</h2>'), 'Headings render');
check(str_contains($html, '<ol>') && str_contains($html, '<li>Record each scene on its own.</li>'), 'Ordered lists render');
check(str_contains($html, '&lt; expected'), 'Code blocks escape HTML');
check(str_contains($html, '<img src="/images/timeline.png" alt="The timeline" loading="lazy">'), 'Images render');

$unsafe = blog_markdown_to_html('<script>alert(1)</script> [bad](javascript:alert(1)) ![x](javascript:alert(1))');
check(!str_contains($unsafe, '<script>'), 'Raw HTML is escaped');
check(!str_contains($unsafe, 'javascript:alert(1)"'), 'Unsafe URLs do not become links or images');

echo "Blog content tests passed\n";
