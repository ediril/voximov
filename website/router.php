<?php

$path = rawurldecode(parse_url($_SERVER['REQUEST_URI'] ?? '/', PHP_URL_PATH) ?: '/');
$routes = ['/' => 'index.php', '/blog' => 'blog.php', '/blog/' => 'blog.php', '/sitemap.xml' => 'sitemap.php', '/feed.xml' => 'feed.php'];

if (preg_match('#^/(_blog_posts|tests)(/|$)#', $path) || $path === '/router.php') {
    require __DIR__ . '/404.php';
    return true;
}
if (isset($routes[$path])) {
    require __DIR__ . '/' . $routes[$path];
    return true;
}
if (is_file(__DIR__ . $path)) {
    return false;
}
if (preg_match('#^/blog/([a-z0-9-]+)/?$#', $path, $matches)) {
    $_GET['slug'] = $matches[1];
    require __DIR__ . '/blog-post.php';
    return true;
}
$page = trim($path, '/');
if (preg_match('/^[a-z0-9-]+$/', $page) && is_file(__DIR__ . '/' . $page . '.php')) {
    require __DIR__ . '/' . $page . '.php';
    return true;
}
require __DIR__ . '/404.php';
return true;
