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
