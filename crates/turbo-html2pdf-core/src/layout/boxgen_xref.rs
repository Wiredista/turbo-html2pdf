//! `xref`-feature box-generation helpers (AC-3.25): deriving named destinations
//! from `<t:anchor>` directives and HTML `id` / `<a name>` attributes, plus
//! internal (`#name`) and external URI link targets from `<a href="…">`.
//! Only compiled under the `xref` feature, so this lives in its own module (kept
//! out of the default coverage surface, like the other gated-only modules).

use crate::node::{Attr, TKind, Tag};
use crate::style::StyledElement;

use super::{display_of, Display, LayoutBox};
use crate::layout::fragment::XrefMeta;

/// The destination name of a `<t:anchor name="x">` (or `id="x"`), or `None`.
pub(super) fn anchor_name(kind: TKind, el: &StyledElement) -> Option<String> {
    if !matches!(kind, TKind::Anchor) {
        return None;
    }
    attr_non_empty(&el.attrs, "name").or_else(|| attr_non_empty(&el.attrs, "id"))
}

/// Whether an HTML element with `display: inline` must be promoted to an atomic
/// inline box so its fragment carries link or anchor metadata through layout.
pub(super) fn is_inline_xref_box(el: &StyledElement) -> bool {
    if !matches!(display_of(&el.style), Display::Inline) {
        return false;
    }
    has_link_href(el) || has_html_anchor(el)
}

/// Cross-reference metadata for a [`LayoutBox`], derived from its HTML attributes.
pub(crate) fn box_xref(lb: &LayoutBox) -> XrefMeta {
    XrefMeta {
        anchor: box_anchor(&lb.attrs),
        link_href: internal_href_from_attrs(&lb.attrs),
        link_uri: external_uri_from_attrs(&lb.attrs),
    }
}

/// Whether `el` is an `<a href="…">` carrying a non-empty internal or external target.
fn has_link_href(el: &StyledElement) -> bool {
    is_html_tag(el, "a")
        && (internal_href_from_attrs(&el.attrs).is_some()
            || external_uri_from_attrs(&el.attrs).is_some())
}

/// Whether `el` carries an HTML destination (`id="…"` on any element, or `name="…"` on `<a>`).
fn has_html_anchor(el: &StyledElement) -> bool {
    if attr_non_empty(&el.attrs, "id").is_some() {
        return true;
    }
    is_html_tag(el, "a") && attr_non_empty(&el.attrs, "name").is_some()
}

/// Whether `el` is the HTML element with tag `expected`.
fn is_html_tag(el: &StyledElement, expected: &str) -> bool {
    matches!(&el.tag, Tag::Html(name) if name == expected)
}

/// The destination name defined by a box's attributes (`id`, or fallback `name`).
fn box_anchor(attrs: &[Attr]) -> Option<String> {
    attr_non_empty(attrs, "id").or_else(|| attr_non_empty(attrs, "name"))
}

/// The bare `#name` destination from an `href="#name"` attribute, or `None`.
fn internal_href_from_attrs(attrs: &[Attr]) -> Option<String> {
    let target = attr_value(attrs, "href")?.trim().strip_prefix('#')?;
    (!target.is_empty()).then(|| target.to_string())
}

/// The external URI from an `href="…"` attribute (non-empty and not starting with `#`).
fn external_uri_from_attrs(attrs: &[Attr]) -> Option<String> {
    let raw = attr_value(attrs, "href")?.trim();
    if raw.is_empty() || raw.starts_with('#') {
        return None;
    }
    Some(raw.to_string())
}

/// The non-empty string value of the named attribute in `attrs`, if present.
fn attr_non_empty(attrs: &[Attr], name: &str) -> Option<String> {
    attr_value(attrs, name)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

/// The value of the named attribute in `attrs`, if present.
fn attr_value<'a>(attrs: &'a [Attr], name: &str) -> Option<&'a str> {
    attrs
        .iter()
        .find(|a| a.name == name)
        .map(|a| a.value.as_str())
}
