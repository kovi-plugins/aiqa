pub static HTML_START_NEXT_IS_MD_CSS: &str = r#"<!doctype html>
<html>
<head>
<meta name="viewport" content="width=device-width, initial-scale=1">
<style>"#;

pub static HTML_2_NEXT_IS_HIGHLIGHT_CSS: &str = r#"
.markdown-body {
    box-sizing: border-box;
    width: 100%;
    max-width: 720px;
    margin: 0;
    padding: 12px 12px 20px 12px;
    height: auto;

    font-family: "MiSans", -apple-system, BlinkMacSystemFont, "Segoe UI", "Noto Sans", Helvetica, Arial, sans-serif, "Apple Color Emoji", "Segoe UI Emoji";
}

body{
    font-family: Arial, sans-serif; /* 选择无衬线字体 */
    margin: 0;
    padding: 0;
    overflow: hidden;
    max-width: 720px;
}
</style>
<style>
"#;

pub static HTML_3_NEXT_IS_MD_BODY_AND_THEN_IS_HTML4: &str = r#"</style>
</head>
<body>
<article class="markdown-body">"#;

pub static HTML_4_NEXT_IS_HIGH_LIGHT_JS: &str = "</article><script>";

pub static HTML_END: &str = r#"</script><script>hljs.highlightAll();</script>
<script>
const elementsToCheck = ['img']; // 需要检测的元素
const timeoutMs = 5000; // 超时，比如 5 秒

document.addEventListener("DOMContentLoaded", function() {
    const markdownBody = document.querySelector('.markdown-body');

    // 找出所有要等待加载的元素
    const elements = [];
    elementsToCheck.forEach(tag => {
        elements.push(...markdownBody.querySelectorAll(tag));
    });

    // 创建等待加载完成的 Promise
    const loadPromises = Array.from(elements).map(el => {
        if (el.complete) {
            // 图片已经加载完毕，立即resolve
            return Promise.resolve();
        } else {
            // 等待图片 load 或 error
            return new Promise(resolve => {
                el.addEventListener('load', resolve, { once: true });
                el.addEventListener('error', resolve, { once: true });
            });
        }
    });

    // 超时 Promise
    const timeoutPromise = new Promise(resolve => {
        setTimeout(resolve, timeoutMs);
    });

    // 等待所有图片加载完成或者超时，哪个先到就执行
    Promise.race([
        Promise.all(loadPromises),
        timeoutPromise
    ]).then(() => {
        markdownBody.style.maxWidth = '720px';

        const finishedElement = document.createElement('div');
        finishedElement.classList.add('finish');
        document.body.appendChild(finishedElement);
    });
});
</script>
</body></html>"#;

#[allow(dead_code)]
pub static HTML_END_FOR_CHANGE: &str = r#"</script><script>hljs.highlightAll();</script>
<script>
const elementsToCheck = ['pre', 'code']; // 需要检测的元素

document.addEventListener("DOMContentLoaded", function() {
    const markdownBody = document.querySelector('.markdown-body');
    let foundElement = false;

    elementsToCheck.forEach(tag => {
        if (markdownBody.querySelector(tag)) {
            foundElement = true;
        }
    });

    if (foundElement) {
        markdownBody.style.maxWidth = '720px';
    } else {
        markdownBody.style.maxWidth = '500px';
    }

    const finishedElement = document.createElement('div');

    finishedElement.classList.add('finish');

    // 完成页面加载
    document.body.appendChild(finishedElement);
});
</script>
</body></html>"#;

pub static HIGH_LIGHT_JS_NEXT_IS_HTML_END: &str = include_str!("html/highlight.js");

pub static HIGH_LIGHT_DARK_CSS_NEXT_IS_HTML3: &str = include_str!("html/highlight_github_dark.css");

pub static HIGH_LIGHT_LIGHT_CSS_NEXT_IS_HTML3: &str =
    include_str!("html/highlight_github_light.css");

pub static GITHUB_MARKDOWN_LIGHT_NEXT_IS_HTML2: &str = include_str!("html/github_md_light.css");

pub static GITHUB_MARKDOWN_DARK_NEXT_IS_HTML2: &str = include_str!("html/github_md_dark.css");
