<?php

const BLOG_POSTS_DIRECTORY = __DIR__ . '/_blog_posts';
const BLOG_SLUG_PATTERN = '/^[a-z0-9]+(?:-[a-z0-9]+)*$/';

function parse_blog_front_matter(string $frontMatter): ?array {
    $metadata = [];
    foreach (explode("\n", $frontMatter) as $line) {
        if (trim($line) === '') {
            continue;
        }
        if (!preg_match('/^([A-Za-z][A-Za-z0-9_-]*):\s*(.*)$/', $line, $matches)) {
            return null;
        }
        $value = trim($matches[2]);
        if (strlen($value) >= 2 && $value[0] === '"' && substr($value, -1) === '"') {
            $value = json_decode($value, true);
            if (!is_string($value)) {
                return null;
            }
        } elseif (strlen($value) >= 2 && $value[0] === "'" && substr($value, -1) === "'") {
            $value = str_replace("''", "'", substr($value, 1, -1));
        }
        $metadata[$matches[1]] = $value;
    }
    return $metadata;
}

function valid_blog_date(string $date): bool {
    $parsed = DateTimeImmutable::createFromFormat('!Y-m-d', $date);
    return $parsed !== false && $parsed->format('Y-m-d') === $date;
}

function parse_blog_post(string $filePath): ?array {
    if (!is_file($filePath) || pathinfo($filePath, PATHINFO_EXTENSION) !== 'md') {
        return null;
    }
    $slug = pathinfo($filePath, PATHINFO_FILENAME);
    if (!preg_match(BLOG_SLUG_PATTERN, $slug)) {
        return null;
    }
    $source = file_get_contents($filePath);
    if ($source === false) {
        return null;
    }
    $source = str_replace("\r\n", "\n", ltrim($source, "\xEF\xBB\xBF"));
    if (!preg_match('/\A---[ \t]*\n(.*?)\n---[ \t]*(?:\n|\z)(.*)\z/s', $source, $matches)) {
        return null;
    }
    $metadata = parse_blog_front_matter($matches[1]);
    $title = trim((string)($metadata['title'] ?? ''));
    $date = trim((string)($metadata['date'] ?? ''));
    if (!$metadata || $title === '' || !valid_blog_date($date)) {
        return null;
    }
    return [
        'title' => $title,
        'date' => $date,
        'excerpt' => trim((string)($metadata['excerpt'] ?? '')),
        'content' => trim($matches[2]),
        'slug' => $slug,
    ];
}

function get_all_blog_posts(string $directory = BLOG_POSTS_DIRECTORY): array {
    $posts = array_values(array_filter(array_map('parse_blog_post', glob($directory . '/*.md') ?: [])));
    usort($posts, static fn(array $left, array $right): int => [$right['date'], $right['slug']] <=> [$left['date'], $left['slug']]);
    return $posts;
}

function get_blog_post_by_slug(string $slug, string $directory = BLOG_POSTS_DIRECTORY): ?array {
    if (!preg_match(BLOG_SLUG_PATTERN, $slug)) {
        return null;
    }
    return parse_blog_post($directory . '/' . $slug . '.md');
}

function format_blog_date(string $date): string {
    $parsed = DateTimeImmutable::createFromFormat('!Y-m-d', $date);
    return $parsed ? $parsed->format('F j, Y') : $date;
}

function safe_blog_url(string $url): bool {
    return (bool)preg_match('#^(?:https?://|mailto:|/|\#)#i', $url);
}

function blog_inline_markdown(string $text): string {
    $tokens = [];
    $keep = static function (string $html) use (&$tokens): string {
        $token = "\x1A" . count($tokens) . "\x1A";
        $tokens[$token] = $html;
        return $token;
    };
    $escape = static fn(string $value): string => htmlspecialchars($value, ENT_QUOTES | ENT_SUBSTITUTE, 'UTF-8');

    $text = preg_replace_callback('/`([^`]+)`/', static fn(array $m): string => $keep('<code>' . $escape($m[1]) . '</code>'), $text);
    $text = preg_replace_callback('/!\[([^\]]*)\]\(([^)\s]+)\)/', static function (array $m) use ($keep, $escape): string {
        if (!safe_blog_url($m[2])) {
            return $m[0];
        }
        return $keep(sprintf('<img src="%s" alt="%s" loading="lazy">', $escape($m[2]), $escape($m[1])));
    }, $text);
    $text = preg_replace_callback('/\[([^\]]+)\]\(([^)\s]+)\)/', static function (array $m) use ($keep, $escape): string {
        if (!safe_blog_url($m[2])) {
            return $m[0];
        }
        return $keep(sprintf('<a href="%s">%s</a>', $escape($m[2]), $escape($m[1])));
    }, $text);

    $text = $escape($text);
    $text = preg_replace('/\*\*([^*]+)\*\*/', '<strong>$1</strong>', $text);
    $text = preg_replace('/__([^_]+)__/', '<strong>$1</strong>', $text);
    $text = preg_replace('/(?<!\w)\*([^*]+)\*(?!\w)/', '<em>$1</em>', $text);
    $text = preg_replace('/(?<!\w)_([^_]+)_(?!\w)/', '<em>$1</em>', $text);
    return strtr($text, $tokens);
}

function blog_code_block(string $language, array $lines): string {
    $class = $language !== '' ? ' class="language-' . htmlspecialchars($language, ENT_QUOTES, 'UTF-8') . '"' : '';
    return '<pre><code' . $class . '>' . htmlspecialchars(implode("\n", $lines), ENT_QUOTES | ENT_SUBSTITUTE, 'UTF-8') . "</code></pre>\n";
}

function blog_markdown_to_html(string $markdown): string {
    $html = '';
    $paragraph = [];
    $listType = null;
    $codeLanguage = null;
    $codeLines = [];

    $closeParagraph = static function () use (&$html, &$paragraph): void {
        if ($paragraph) {
            $html .= '<p>' . blog_inline_markdown(implode(' ', $paragraph)) . "</p>\n";
            $paragraph = [];
        }
    };
    $closeList = static function () use (&$html, &$listType): void {
        if ($listType !== null) {
            $html .= "</{$listType}>\n";
            $listType = null;
        }
    };
    $closeBlocks = static function () use ($closeParagraph, $closeList): void {
        $closeParagraph();
        $closeList();
    };
    $listItem = static function (string $type, string $text) use (&$html, &$listType, $closeParagraph, $closeList): void {
        $closeParagraph();
        if ($listType !== $type) {
            $closeList();
            $html .= "<{$type}>\n";
            $listType = $type;
        }
        $html .= '<li>' . blog_inline_markdown($text) . "</li>\n";
    };

    foreach (explode("\n", str_replace("\r\n", "\n", $markdown)) as $line) {
        $trimmed = trim($line);
        if ($codeLanguage !== null) {
            if (preg_match('/^```\s*$/', $trimmed)) {
                $html .= blog_code_block($codeLanguage, $codeLines);
                $codeLanguage = null;
                $codeLines = [];
            } else {
                $codeLines[] = $line;
            }
        } elseif (preg_match('/^```([A-Za-z0-9_-]*)\s*$/', $trimmed, $matches)) {
            $closeBlocks();
            $codeLanguage = $matches[1];
        } elseif ($trimmed === '') {
            $closeBlocks();
        } elseif (preg_match('/^(#{1,6})\s+(.+)$/', $trimmed, $matches)) {
            $closeBlocks();
            $level = strlen($matches[1]);
            $html .= "<h{$level}>" . blog_inline_markdown($matches[2]) . "</h{$level}>\n";
        } elseif (preg_match('/^(?:---+|___+|\*\*\*+)$/', $trimmed)) {
            $closeBlocks();
            $html .= "<hr>\n";
        } elseif (preg_match('/^>\s?(.*)$/', $trimmed, $matches)) {
            $closeBlocks();
            $html .= '<blockquote><p>' . blog_inline_markdown($matches[1]) . "</p></blockquote>\n";
        } elseif (preg_match('/^[-*]\s+(.+)$/', $trimmed, $matches)) {
            $listItem('ul', $matches[1]);
        } elseif (preg_match('/^\d+\.\s+(.+)$/', $trimmed, $matches)) {
            $listItem('ol', $matches[1]);
        } else {
            $closeList();
            $paragraph[] = $trimmed;
        }
    }

    if ($codeLanguage !== null) {
        $html .= blog_code_block($codeLanguage, $codeLines);
    }
    $closeBlocks();
    return $html;
}
