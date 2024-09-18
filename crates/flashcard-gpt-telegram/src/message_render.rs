use std::cmp::Ordering;
use std::fmt::Write;
use teloxide::prelude::Message;
use teloxide::types::{MessageEntity, MessageEntityKind as MEK};

#[derive(Clone)]
pub struct Tag<'a> {
    pub place: Place,
    pub kind: Kind<'a>,
    pub offset: usize,
    pub index: usize,
}

impl<'a> Tag<'a> {
    #[inline(always)]
    pub const fn start(kind: Kind<'a>, offset: usize, index: usize) -> Self {
        Self {
            place: Place::Start,
            kind,
            offset,
            index,
        }
    }

    #[inline(always)]
    pub const fn end(kind: Kind<'a>, offset: usize, index: usize) -> Self {
        Self {
            place: Place::End,
            kind,
            offset,
            index,
        }
    }
}

impl<'a> Eq for Tag<'a> {}

impl<'a> PartialEq for Tag<'a> {
    fn eq(&self, other: &Self) -> bool {
        // We don't check kind here
        self.place == other.place && self.offset == other.offset && self.index == other.index
    }
}

impl<'a> Ord for Tag<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.offset
            .cmp(&other.offset)
            .then_with(|| self.place.cmp(&other.place))
            .then_with(|| match other.place {
                Place::Start => self.index.cmp(&other.index),
                Place::End => other.index.cmp(&self.index),
            })
    }
}

impl<'a> PartialOrd for Tag<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Place {
    // HACK: `End` needs to be first because of the `Ord` Implementation.
    // the reason is when comparing tags we want the `End` to be first if the offset
    // is the same.
    End,
    Start,
}

#[derive(Clone, PartialEq, Eq)]
pub enum Kind<'a> {
    Bold,
    Blockquote,
    Italic,
    Underline,
    Strikethrough,
    Spoiler,
    Code,
    Pre(Option<&'a str>),
    TextLink(&'a str),
    TextMention(u64),
    CustomEmoji(&'a str),
}

pub struct SimpleTag {
    pub start: &'static str,
    pub end: &'static str,
}

impl SimpleTag {
    #[inline]
    pub const fn new(start: &'static str, end: &'static str) -> Self {
        Self { start, end }
    }

    /// Get tag size based on place
    pub const fn get_tag(&self, place: Place) -> &'static str {
        match place {
            Place::Start => self.start,
            Place::End => self.end,
        }
    }
}

pub struct ComplexTag {
    pub start: &'static str,
    pub middle: &'static str,
    pub end: &'static str,
}

impl ComplexTag {
    #[inline]
    pub const fn new(start: &'static str, middle: &'static str, end: &'static str) -> Self {
        Self { start, middle, end }
    }
}

pub struct TagWriter {
    pub bold: SimpleTag,
    pub blockquote: SimpleTag,
    pub italic: SimpleTag,
    pub underline: SimpleTag,
    pub strikethrough: SimpleTag,
    pub spoiler: SimpleTag,
    pub code: SimpleTag,
    pub pre_no_lang: SimpleTag,
    pub pre: ComplexTag,
    pub text_link: ComplexTag,
    pub text_mention: ComplexTag,
    pub custom_emoji: ComplexTag,

    /// Write the tag to buffer
    pub write_tag_fn: fn(&Tag, buf: &mut String),
    /// Write the char to buffer and escape characters if needed
    pub write_char_fn: fn(char, buf: &mut String),
}

impl TagWriter {
    /// Get the extra size needed for tags
    pub fn get_tags_sizes(&self, tags: &[Tag]) -> usize {
        tags.iter()
            .map(|tag| match tag.kind {
                Kind::Bold => self.bold.get_tag(tag.place).len(),
                Kind::Blockquote => self.blockquote.get_tag(tag.place).len(),
                Kind::Italic => self.italic.get_tag(tag.place).len(),
                Kind::Underline => self.underline.get_tag(tag.place).len(),
                Kind::Strikethrough => self.strikethrough.get_tag(tag.place).len(),
                Kind::Spoiler => self.spoiler.get_tag(tag.place).len(),
                Kind::Code => self.code.get_tag(tag.place).len(),
                Kind::Pre(lang) => match tag.place {
                    Place::Start => lang.map_or(self.pre_no_lang.start.len(), |l| {
                        self.pre.start.len() + l.len()
                    }),
                    Place::End => lang.map_or(self.pre_no_lang.end.len(), |_| {
                        self.pre.middle.len() + self.pre.end.len()
                    }),
                },
                Kind::TextLink(url) => match tag.place {
                    Place::Start => self.text_link.start.len() + url.len(),
                    Place::End => self.text_link.middle.len() + self.text_link.end.len(),
                },
                Kind::TextMention(id) => match tag.place {
                    Place::Start => self.text_mention.start.len() + id.ilog10() as usize + 1,
                    Place::End => self.text_mention.middle.len() + self.text_mention.end.len(),
                },
                Kind::CustomEmoji(custom_emoji_id) => match tag.place {
                    Place::Start => self.custom_emoji.start.len() + custom_emoji_id.len(),
                    Place::End => self.custom_emoji.middle.len() + self.custom_emoji.end.len(),
                },
            })
            .sum()
    }
}

pub static HTML: TagWriter = TagWriter {
    bold: SimpleTag::new("<b>", "</b>"),
    blockquote: SimpleTag::new("<blockquote>", "</blockquote>"),
    italic: SimpleTag::new("<i>", "</i>"),
    underline: SimpleTag::new("<u>", "</u>"),
    strikethrough: SimpleTag::new("<s>", "</s>"),
    spoiler: SimpleTag::new("<tg-spoiler>", "</tg-spoiler>"),
    code: SimpleTag::new("<code>", "</code>"),
    pre_no_lang: SimpleTag::new("<pre>", "</pre>"),
    pre: ComplexTag::new("<pre><code class=\"language-", "\">", "</code></pre>"),
    text_link: ComplexTag::new("<a href=\"", "\">", "</a>"),
    text_mention: ComplexTag::new("<a href=\"tg://user?id=", "\">", "</a>"),
    custom_emoji: ComplexTag::new("<tg-emoji emoji-id=\"", "\">", "</tg-emoji>"),

    write_tag_fn: write_tag,
    write_char_fn: write_char,
};

fn write_tag(tag: &Tag, buf: &mut String) {
    match tag.kind {
        Kind::Bold => buf.push_str(HTML.bold.get_tag(tag.place)),
        Kind::Blockquote => buf.push_str(HTML.blockquote.get_tag(tag.place)),
        Kind::Italic => buf.push_str(HTML.italic.get_tag(tag.place)),
        Kind::Underline => buf.push_str(HTML.underline.get_tag(tag.place)),
        Kind::Strikethrough => buf.push_str(HTML.strikethrough.get_tag(tag.place)),
        Kind::Spoiler => buf.push_str(HTML.spoiler.get_tag(tag.place)),
        Kind::Code => buf.push_str(HTML.code.get_tag(tag.place)),
        Kind::Pre(lang) => match tag.place {
            Place::Start => match lang {
                Some(lang) => write!(buf, "{}{}{}", HTML.pre.start, lang, HTML.pre.middle).unwrap(),
                None => buf.push_str(HTML.pre_no_lang.start),
            },
            Place::End => buf.push_str(lang.map_or(HTML.pre_no_lang.end, |_| HTML.pre.end)),
        },
        Kind::TextLink(url) => match tag.place {
            Place::Start => write!(
                buf,
                "{}{}{}",
                HTML.text_link.start, url, HTML.text_link.middle
            )
            .unwrap(),
            Place::End => buf.push_str(HTML.text_link.end),
        },
        Kind::TextMention(id) => match tag.place {
            Place::Start => write!(
                buf,
                "{}{}{}",
                HTML.text_mention.start, id, HTML.text_mention.middle
            )
            .unwrap(),
            Place::End => buf.push_str(HTML.text_mention.end),
        },
        Kind::CustomEmoji(custom_emoji_id) => match tag.place {
            Place::Start => write!(
                buf,
                "{}{}{}",
                HTML.custom_emoji.start, custom_emoji_id, HTML.custom_emoji.middle
            )
            .unwrap(),
            Place::End => buf.push_str(HTML.custom_emoji.end),
        },
    }
}

fn write_char(ch: char, buf: &mut String) {
    match ch {
        '&' => buf.push_str("&amp;"),
        '<' => buf.push_str("&lt;"),
        '>' => buf.push_str("&gt;"),
        c => buf.push(c),
    }
}

/// The [`RenderMessageTextHelper`] trait provides methods to generate HTML and
/// Markdown representations of the text and captions in a Telegram message.
pub trait RenderMessageTextHelper {
    /// Returns the HTML representation of the message text, if the message
    /// contains text. This method will parse the text and any entities
    /// (such as bold, italic, links, etc.) and return the HTML-formatted
    /// string.
    #[must_use]
    fn html_text(&self) -> Option<String>;
    /// Returns the Markdown representation of the message text, if the message
    /// contains text. This method will parse the text and any entities
    /// (such as bold, italic, links, etc.) and return the
    /// Markdown-formatted string.

    #[must_use]
    fn html_caption(&self) -> Option<String>;
}

impl RenderMessageTextHelper for Message {
    fn html_text(&self) -> Option<String> {
        self.text()
            .zip(self.entities())
            .map(|(text, entities)| Render::new(text, entities).as_html())
    }

    fn html_caption(&self) -> Option<String> {
        self.caption()
            .zip(self.caption_entities())
            .map(|(text, entities)| Render::new(text, entities).as_html())
    }
}

/// The [`Render`] struct is responsible for parsing the text and entities to
/// produce the final formatted output.
#[derive(Clone, Eq, PartialEq)]
pub struct Render<'a> {
    text: &'a str,
    tags: Vec<Tag<'a>>,
}

impl<'a> Render<'a> {
    /// Creates a new `Render` instance with the given text and entities.
    ///
    /// The `Render` is responsible for parsing the text and entities to
    /// produce the final formatted output. This constructor sets up the
    /// initial state needed for the parsing process.
    ///
    /// # Arguments
    ///
    /// - `text`: The input text to be parsed.
    /// - `entities`: The message entities (formatting, links, etc.) to be
    ///   applied to the text.
    ///
    /// # Returns
    ///
    /// A new [`Render`] instance.
    #[must_use]
    pub fn new(text: &'a str, entities: &'a [MessageEntity]) -> Self {
        // get the needed size for the new tags that we want to parse from entities
        let needed_size: usize = entities
            .iter()
            .filter(|e| {
                matches!(
                    e.kind,
                    MEK::Bold
                        | MEK::Blockquote
                        | MEK::Italic
                        | MEK::Underline
                        | MEK::Strikethrough
                        | MEK::Spoiler
                        | MEK::Code
                        | MEK::Pre { .. }
                        | MEK::TextLink { .. }
                        | MEK::TextMention { .. }
                        | MEK::CustomEmoji { .. }
                )
            })
            .count()
            * 2; // 2 because we inseret two tag for each entity

        let mut tags = Vec::with_capacity(needed_size);

        for (index, entity) in entities.iter().enumerate() {
            let kind = match &entity.kind {
                MEK::Bold => Kind::Bold,
                MEK::Blockquote => Kind::Blockquote,
                MEK::Italic => Kind::Italic,
                MEK::Underline => Kind::Underline,
                MEK::Strikethrough => Kind::Strikethrough,
                MEK::Spoiler => Kind::Spoiler,
                MEK::Code => Kind::Code,
                MEK::Pre { language } => Kind::Pre(language.as_ref().map(String::as_str)),
                MEK::TextLink { url } => Kind::TextLink(url.as_str()),
                MEK::TextMention { user } => Kind::TextMention(user.id.0),
                MEK::CustomEmoji { custom_emoji_id } => Kind::CustomEmoji(custom_emoji_id),
                _ => continue,
            };

            // FIXME: maybe instead of clone store all the `kind`s in a seperate
            // vector and then just store the index here?
            tags.push(Tag::start(kind.clone(), entity.offset, index));
            tags.push(Tag::end(kind, entity.offset + entity.length, index));
        }

        tags.sort_unstable();

        Self { text, tags }
    }

    /// Renders the text with the given [`TagWriter`].
    ///
    /// This method iterates through the text and the associated position tags,
    /// and writes the text with the appropriate tags to a buffer. The
    /// resulting buffer is then returned as a `String`.
    ///
    /// If input have no tags we just return the original text as-is.
    #[must_use]
    fn format(&self, writer: &TagWriter) -> String {
        if self.tags.is_empty() {
            return self.text.to_owned();
        }

        let mut buffer = String::with_capacity(self.text.len() + writer.get_tags_sizes(&self.tags));
        let mut tags = self.tags.iter();
        let mut current_tag = tags.next();

        let mut prev_point = None;

        for (idx, point) in self.text.encode_utf16().enumerate() {
            loop {
                match current_tag {
                    Some(tag) if tag.offset == idx => {
                        (writer.write_tag_fn)(tag, &mut buffer);
                        current_tag = tags.next();
                    }
                    _ => break,
                }
            }

            let ch = if let Some(previous) = prev_point.take() {
                char::decode_utf16([previous, point])
                    .next()
                    .unwrap()
                    .unwrap()
            } else {
                match char::decode_utf16([point]).next().unwrap() {
                    Ok(c) => c,
                    Err(unpaired) => {
                        prev_point = Some(unpaired.unpaired_surrogate());
                        continue;
                    }
                }
            };

            (writer.write_char_fn)(ch, &mut buffer);
        }

        for tag in current_tag.into_iter().chain(tags) {
            (writer.write_tag_fn)(tag, &mut buffer);
        }

        buffer
    }

    /// Render and return the text as **Html-formatted** string.
    #[must_use]
    #[inline]
    pub fn as_html(&self) -> String {
        self.format(&HTML)
    }
}
