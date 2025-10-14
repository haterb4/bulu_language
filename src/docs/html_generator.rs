//! HTML documentation generator

use crate::Result;
use crate::project::Project;
use super::{DocumentedItem, ItemKind, Visibility};
use std::path::{Path, PathBuf};
use std::fs;
// HashMap not needed currently

/// Generates HTML documentation
pub struct HtmlGenerator {
    output_dir: PathBuf,
}

impl HtmlGenerator {
    pub fn new(output_dir: &Path) -> Self {
        Self {
            output_dir: output_dir.to_path_buf(),
        }
    }

    /// Generate HTML documentation
    pub fn generate(&self, items: &[DocumentedItem], project: &Project) -> Result<()> {
        // Create necessary directories
        fs::create_dir_all(&self.output_dir)?;
        fs::create_dir_all(self.output_dir.join("static"))?;

        // Generate CSS
        self.generate_css()?;

        // Generate JavaScript
        self.generate_js()?;

        // Generate index page
        self.generate_index(items, project)?;

        // Generate individual pages for each item
        self.generate_item_pages(items, project)?;

        Ok(())
    }

    fn generate_css(&self) -> Result<()> {
        let css = r#"
/* Bulu Documentation Styles */
:root {
    --primary-color: #2563eb;
    --secondary-color: #64748b;
    --background-color: #ffffff;
    --surface-color: #f8fafc;
    --text-color: #1e293b;
    --text-muted: #64748b;
    --border-color: #e2e8f0;
    --code-background: #f1f5f9;
    --success-color: #059669;
    --warning-color: #d97706;
    --error-color: #dc2626;
}

* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    line-height: 1.6;
    color: var(--text-color);
    background-color: var(--background-color);
}

.container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 0 20px;
}

/* Header */
.header {
    background-color: var(--surface-color);
    border-bottom: 1px solid var(--border-color);
    padding: 1rem 0;
}

.header h1 {
    color: var(--primary-color);
    font-size: 2rem;
    font-weight: 700;
}

.header p {
    color: var(--text-muted);
    margin-top: 0.5rem;
}

/* Navigation */
.nav {
    background-color: var(--background-color);
    border-bottom: 1px solid var(--border-color);
    padding: 1rem 0;
    position: sticky;
    top: 0;
    z-index: 100;
}

.nav ul {
    list-style: none;
    display: flex;
    gap: 2rem;
}

.nav a {
    color: var(--text-color);
    text-decoration: none;
    font-weight: 500;
    padding: 0.5rem 1rem;
    border-radius: 0.375rem;
    transition: background-color 0.2s;
}

.nav a:hover {
    background-color: var(--surface-color);
}

.nav a.active {
    background-color: var(--primary-color);
    color: white;
}

/* Main content */
.main {
    padding: 2rem 0;
}

.sidebar {
    width: 250px;
    position: fixed;
    top: 120px;
    left: 20px;
    height: calc(100vh - 140px);
    overflow-y: auto;
    background-color: var(--surface-color);
    border-radius: 0.5rem;
    padding: 1rem;
}

.content {
    margin-left: 290px;
    padding: 0 2rem;
}

/* Sidebar */
.sidebar h3 {
    color: var(--primary-color);
    font-size: 1.1rem;
    margin-bottom: 0.5rem;
    margin-top: 1rem;
}

.sidebar h3:first-child {
    margin-top: 0;
}

.sidebar ul {
    list-style: none;
}

.sidebar li {
    margin-bottom: 0.25rem;
}

.sidebar a {
    color: var(--text-color);
    text-decoration: none;
    font-size: 0.9rem;
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
    display: block;
    transition: background-color 0.2s;
}

.sidebar a:hover {
    background-color: var(--background-color);
}

/* Item documentation */
.item {
    margin-bottom: 3rem;
    padding-bottom: 2rem;
    border-bottom: 1px solid var(--border-color);
}

.item:last-child {
    border-bottom: none;
}

.item-header {
    margin-bottom: 1rem;
}

.item-title {
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--primary-color);
    margin-bottom: 0.5rem;
}

.item-kind {
    display: inline-block;
    background-color: var(--primary-color);
    color: white;
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    font-weight: 500;
    text-transform: uppercase;
    margin-right: 0.5rem;
}

.item-visibility {
    display: inline-block;
    background-color: var(--secondary-color);
    color: white;
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    font-weight: 500;
    text-transform: uppercase;
}

.item-signature {
    background-color: var(--code-background);
    border: 1px solid var(--border-color);
    border-radius: 0.375rem;
    padding: 1rem;
    margin: 1rem 0;
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    font-size: 0.9rem;
    overflow-x: auto;
}

.item-description {
    margin: 1rem 0;
    line-height: 1.7;
}

.item-section {
    margin: 1.5rem 0;
}

.item-section h4 {
    color: var(--primary-color);
    font-size: 1.1rem;
    margin-bottom: 0.5rem;
}

.param-list {
    list-style: none;
}

.param-list li {
    margin-bottom: 0.5rem;
    padding: 0.5rem;
    background-color: var(--surface-color);
    border-radius: 0.25rem;
}

.param-name {
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    font-weight: 600;
    color: var(--primary-color);
}

.example {
    background-color: var(--code-background);
    border: 1px solid var(--border-color);
    border-radius: 0.375rem;
    padding: 1rem;
    margin: 1rem 0;
    font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
    font-size: 0.9rem;
    overflow-x: auto;
}

/* Search */
.search {
    margin-bottom: 1rem;
}

.search input {
    width: 100%;
    padding: 0.5rem;
    border: 1px solid var(--border-color);
    border-radius: 0.375rem;
    font-size: 0.9rem;
}

.search input:focus {
    outline: none;
    border-color: var(--primary-color);
    box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.1);
}

/* Responsive */
@media (max-width: 768px) {
    .sidebar {
        display: none;
    }
    
    .content {
        margin-left: 0;
        padding: 0 1rem;
    }
}

/* Deprecated items */
.deprecated {
    opacity: 0.7;
}

.deprecated .item-title::after {
    content: " (deprecated)";
    color: var(--warning-color);
    font-size: 0.8rem;
    font-weight: normal;
}

/* Utility classes */
.text-muted {
    color: var(--text-muted);
}

.text-small {
    font-size: 0.875rem;
}

.mt-1 { margin-top: 0.25rem; }
.mt-2 { margin-top: 0.5rem; }
.mt-3 { margin-top: 0.75rem; }
.mt-4 { margin-top: 1rem; }

.mb-1 { margin-bottom: 0.25rem; }
.mb-2 { margin-bottom: 0.5rem; }
.mb-3 { margin-bottom: 0.75rem; }
.mb-4 { margin-bottom: 1rem; }
"#;

        fs::write(self.output_dir.join("static").join("style.css"), css)?;
        Ok(())
    }

    fn generate_js(&self) -> Result<()> {
        let js = "
// Bulu Documentation JavaScript

document.addEventListener('DOMContentLoaded', function() {
    // Search functionality
    const searchInput = document.getElementById('search');
    if (searchInput) {
        searchInput.addEventListener('input', function() {
            const query = this.value.toLowerCase();
            const items = document.querySelectorAll('.item');
            
            items.forEach(item => {
                const title = item.querySelector('.item-title').textContent.toLowerCase();
                const description = item.querySelector('.item-description')?.textContent.toLowerCase() || '';
                
                if (title.includes(query) || description.includes(query)) {
                    item.style.display = 'block';
                } else {
                    item.style.display = 'none';
                }
            });
        });
    }

    // Sidebar navigation
    const sidebarLinks = document.querySelectorAll('.sidebar a');
    sidebarLinks.forEach(link => {
        link.addEventListener('click', function(e) {
            e.preventDefault();
            const targetId = this.getAttribute('href').substring(1);
            const targetElement = document.getElementById(targetId);
            
            if (targetElement) {
                targetElement.scrollIntoView({ behavior: 'smooth' });
            }
        });
    });

    // Highlight current section in sidebar
    const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                const id = entry.target.id;
                const selector = '.sidebar a[href=\"#' + id + '\"]';
                const activeLink = document.querySelector(selector);
                
                // Remove active class from all links
                sidebarLinks.forEach(link => link.classList.remove('active'));
                
                // Add active class to current link
                if (activeLink) {
                    activeLink.classList.add('active');
                }
            }
        });
    }, { threshold: 0.5 });

    // Observe all items
    document.querySelectorAll('.item').forEach(item => {
        observer.observe(item);
    });
});
";

        fs::write(self.output_dir.join("static").join("script.js"), js)?;
        Ok(())
    }

    fn generate_index(&self, items: &[DocumentedItem], project: &Project) -> Result<()> {
        let mut html = String::new();
        
        // HTML header
        html.push_str(&self.generate_html_header(&format!("{} Documentation", project.config.package.name)));
        
        // Body start
        html.push_str("<body>\n");
        
        // Header
        html.push_str(&format!("
<div class=\"header\">
    <div class=\"container\">
        <h1>{}</h1>
        <p>API Documentation</p>
    </div>
</div>
", project.config.package.name));

        // Navigation
        html.push_str("
<div class=\"nav\">
    <div class=\"container\">
        <ul>
            <li><a href=\"#functions\" class=\"active\">Functions</a></li>
            <li><a href=\"#structs\">Structs</a></li>
            <li><a href=\"#interfaces\">Interfaces</a></li>
            <li><a href=\"#constants\">Constants</a></li>
        </ul>
    </div>
</div>
");

        // Main content
        html.push_str("
<div class=\"main\">
    <div class=\"container\">
        <div class=\"sidebar\">
            <div class=\"search\">
                <input type=\"text\" id=\"search\" placeholder=\"Search documentation...\">
            </div>
");

        // Generate sidebar
        self.generate_sidebar(&mut html, items);
        
        html.push_str("
        </div>
        <div class=\"content\">
");

        // Generate content sections
        self.generate_content_sections(&mut html, items);
        
        html.push_str("
        </div>
    </div>
</div>
");

        // HTML footer
        html.push_str(&self.generate_html_footer());
        
        fs::write(self.output_dir.join("index.html"), html)?;
        Ok(())
    }

    fn generate_sidebar(&self, html: &mut String, items: &[DocumentedItem]) {
        // Group items by kind
        let mut functions = Vec::new();
        let mut structs = Vec::new();
        let mut interfaces = Vec::new();
        let mut constants = Vec::new();

        for item in items {
            if matches!(item.visibility, Visibility::Public) {
                match item.kind {
                    ItemKind::Function => functions.push(item),
                    ItemKind::Struct => structs.push(item),
                    ItemKind::Interface => interfaces.push(item),
                    ItemKind::Constant => constants.push(item),
                    _ => {}
                }
            }
        }

        if !functions.is_empty() {
            html.push_str("<h3>Functions</h3>\n<ul>\n");
            for func in functions {
                html.push_str(&format!("<li><a href=\"#{}\">{}</a></li>", 
                    self.sanitize_id(&func.name), func.name));
            }
            html.push_str("</ul>\n");
        }

        if !structs.is_empty() {
            html.push_str("<h3>Structs</h3>\n<ul>\n");
            for struct_item in structs {
                html.push_str(&format!("<li><a href=\"#{}\">{}</a></li>", 
                    self.sanitize_id(&struct_item.name), struct_item.name));
            }
            html.push_str("</ul>\n");
        }

        if !interfaces.is_empty() {
            html.push_str("<h3>Interfaces</h3>\n<ul>\n");
            for interface in interfaces {
                html.push_str(&format!("<li><a href=\"#{}\">{}</a></li>", 
                    self.sanitize_id(&interface.name), interface.name));
            }
            html.push_str("</ul>\n");
        }

        if !constants.is_empty() {
            html.push_str("<h3>Constants</h3>\n<ul>\n");
            for constant in constants {
                html.push_str(&format!("<li><a href=\"#{}\">{}</a></li>", 
                    self.sanitize_id(&constant.name), constant.name));
            }
            html.push_str("</ul>\n");
        }
    }

    fn generate_content_sections(&self, html: &mut String, items: &[DocumentedItem]) {
        // Group items by kind
        let mut functions = Vec::new();
        let mut structs = Vec::new();
        let mut interfaces = Vec::new();
        let mut constants = Vec::new();

        for item in items {
            if matches!(item.visibility, Visibility::Public) {
                match item.kind {
                    ItemKind::Function => functions.push(item),
                    ItemKind::Struct => structs.push(item),
                    ItemKind::Interface => interfaces.push(item),
                    ItemKind::Constant => constants.push(item),
                    _ => {}
                }
            }
        }

        if !functions.is_empty() {
            html.push_str("<section id=\"functions\"><h2>Functions</h2>");
            for func in functions {
                self.generate_item_html(html, func);
            }
            html.push_str("</section>\n");
        }

        if !structs.is_empty() {
            html.push_str("<section id=\"structs\"><h2>Structs</h2>");
            for struct_item in structs {
                self.generate_item_html(html, struct_item);
            }
            html.push_str("</section>\n");
        }

        if !interfaces.is_empty() {
            html.push_str("<section id=\"interfaces\"><h2>Interfaces</h2>");
            for interface in interfaces {
                self.generate_item_html(html, interface);
            }
            html.push_str("</section>\n");
        }

        if !constants.is_empty() {
            html.push_str("<section id=\"constants\"><h2>Constants</h2>");
            for constant in constants {
                self.generate_item_html(html, constant);
            }
            html.push_str("</section>\n");
        }
    }

    fn generate_item_html(&self, html: &mut String, item: &DocumentedItem) {
        let deprecated_class = if item.doc_comment.as_ref()
            .map(|doc| doc.deprecated.is_some())
            .unwrap_or(false) { " deprecated" } else { "" };

        html.push_str(&format!("
<div class=\"item{}\" id=\"{}\">
    <div class=\"item-header\">
        <h3 class=\"item-title\">{}</h3>
        <span class=\"item-kind\">{:?}</span>
        <span class=\"item-visibility\">{:?}</span>
    </div>
    <div class=\"item-signature\">{}</div>
", 
            deprecated_class,
            self.sanitize_id(&item.name),
            self.escape_html(&item.name),
            item.kind,
            item.visibility,
            self.escape_html(&item.signature)
        ));

        if let Some(doc) = &item.doc_comment {
            if !doc.content.is_empty() {
                html.push_str(&format!("
    <div class=\"item-description\">{}</div>
", self.markdown_to_html(&doc.content)));
            }

            if !doc.params.is_empty() {
                html.push_str("
    <div class=\"item-section\">
        <h4>Parameters</h4>
        <ul class=\"param-list\">
");
                for (param, desc) in &doc.params {
                    html.push_str(&format!("
            <li><span class=\"param-name\">{}</span>: {}</li>
", self.escape_html(param), self.escape_html(desc)));
                }
                html.push_str("        </ul>\n    </div>\n");
            }

            if let Some(returns) = &doc.returns {
                html.push_str(&format!("
    <div class=\"item-section\">
        <h4>Returns</h4>
        <p>{}</p>
    </div>
", self.escape_html(returns)));
            }

            if !doc.examples.is_empty() {
                html.push_str("
    <div class=\"item-section\">
        <h4>Examples</h4>
");
                for example in &doc.examples {
                    html.push_str(&format!("
        <div class=\"example\">{}</div>
", self.escape_html(example)));
                }
                html.push_str("    </div>\n");
            }

            if let Some(deprecated) = &doc.deprecated {
                html.push_str(&format!("
    <div class=\"item-section\">
        <h4 style=\"color: var(--warning-color);\">Deprecated</h4>
        <p>{}</p>
    </div>
", self.escape_html(deprecated)));
            }
        }

        html.push_str("</div>\n");
    }

    fn generate_item_pages(&self, items: &[DocumentedItem], project: &Project) -> Result<()> {
        // For now, we'll just generate the main index page
        // Individual item pages can be added later if needed
        Ok(())
    }

    fn generate_html_header(&self, title: &str) -> String {
        format!("<!DOCTYPE html>
<html lang=\"en\">
<head>
    <meta charset=\"UTF-8\">
    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">
    <title>{}</title>
    <link rel=\"stylesheet\" href=\"static/style.css\">
</head>
", self.escape_html(title))
    }

    fn generate_html_footer(&self) -> String {
        "
<script src=\"static/script.js\"></script>
</body>
</html>
".to_string()
    }

    fn escape_html(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
    }

    fn sanitize_id(&self, text: &str) -> String {
        text.chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
            .collect::<String>()
            .to_lowercase()
    }

    fn markdown_to_html(&self, text: &str) -> String {
        // Simple markdown to HTML conversion
        // This is a basic implementation - a full markdown parser would be better
        let mut html = self.escape_html(text);
        
        // Convert **bold** to <strong>
        html = html.replace("**", "<strong>").replace("</strong>", "</strong>");
        
        // Convert *italic* to <em>
        html = html.replace("*", "<em>").replace("</em>", "</em>");
        
        // Convert `code` to <code>
        html = html.replace("`", "<code>").replace("</code>", "</code>");
        
        // Convert newlines to <br>
        html = html.replace('\n', "<br>");
        
        html
    }
}