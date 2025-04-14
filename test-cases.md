# Test Cases for Markdown Checker

## 1. Link Tests

### Valid Links
- [Rust Book](https://doc.rust-lang.org/book)
- [Mozilla Developer](https://developer.mozilla.org)
- [GitHub](https://github.com)
- [IANA Reserved Domains](http://res-dom.iana.org/)
- [IANA Reserved Domains](https://www.iana.org/domains/reserved)
- [Google](http://google.com/)

### Redirect Links
- [IANA Reserved Domains](http://res-dom.iana.org/)
- [IANA Reserved Domains](https://www.iana.org/domains/reserved)
- [Google](http://google.com/)
- [Domain without www](http://example.com)
- [Domain with www](http://www.example.com)

### Invalid Links
- [Broken Link](https://this-is-invalid-url-for-testing.com)
- [Empty Link]()
- []()
- [Missing Protocol](example.com)

## 2. HTML Tests

### HTML in Markdown
<div class="test">
  This is HTML content
</div>

<p>Another HTML paragraph</p>

## 3. Long Lines Test

This is a very long line that should exceed the maximum line length limit and trigger the long lines rule. It contains more than 100 characters to test the rule.

## 4. Image Tests

### Images with Alt Text
![Rust Logo](https://www.rust-lang.org/logos/rust-logo-256x256.png "Rust Programming Language")

### Images without Alt Text
![][https://example.com/image.jpg]
![](https://example.com/another-image.jpg)

## 5. Mixed Cases

- [Valid Link with <HTML>](https://example.com)
- <a href="https://example.com">HTML Link</a>
- ![](https://example.com/image.jpg "Has title but no alt")

## 6. Special Cases

### Multiple Links in One Line
Here are [multiple](https://example1.com) [links](https://example2.com) in one line.

### Autolinks
<https://auto-link.com>
<invalid-auto-link>

### Code Blocks
```html
<div>HTML in code block should not trigger HTML rule</div>



