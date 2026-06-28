//! Unified mdBook runtime data and chapter traversal helpers.
use mdbook_preprocessor::book::{Book, BookItem, Chapter};
use rayon::prelude::*;

use super::*;

/// Runtime wrapper for mdBook context, config and book state.
pub struct MdBookRuntime<'a> {
    ctx: &'a PreprocessorContext,
    cfg: KatexConfig,
    book: Book,
}

impl<'a> MdBookRuntime<'a> {
    /// Build a runtime wrapper from mdBook inputs.
    pub fn new(ctx: &'a PreprocessorContext, book: Book) -> Result<Self> {
        Ok(Self {
            cfg: get_config(&ctx.config)?,
            ctx,
            book,
        })
    }

    /// Access parsed config.
    pub fn cfg(&self) -> &KatexConfig {
        &self.cfg
    }

    /// Access mdBook context.
    pub fn ctx(&self) -> &PreprocessorContext {
        self.ctx
    }

    /// Inject stylesheet header based on config.
    pub fn stylesheet_header(&self) -> String {
        if self.cfg.no_css {
            String::new()
        } else {
            KATEX_HEADER.to_owned()
        }
    }

    /// Emit version compatibility warning.
    pub fn emit_compatibility_warning(&self) {
        if self.ctx.mdbook_version != mdbook_preprocessor::MDBOOK_VERSION {
            warn!(
                "This mdbook-katex was built against mdbook v{}, but we are being called from mdbook v{}. If you have any issue, this might be a reason.",
                mdbook_preprocessor::MDBOOK_VERSION,
                self.ctx.mdbook_version,
            );
        }
    }

    /// Process all chapters with a unified traversal pipeline.
    pub fn map_chapters<F>(&mut self, handler: F)
    where
        F: Fn(&Chapter) -> String + Sync + Send,
    {
        map_book_chapters(&mut self.book.items, &handler);
    }

    /// Process all chapters in parallel with stable output ordering.
    pub fn map_chapters_parallel<F>(&mut self, handler: F)
    where
        F: Fn(&Chapter) -> String + Sync + Send,
    {
        map_book_chapters_parallel(&mut self.book.items, &handler);
    }

    /// Extract processed book.
    pub fn into_book(self) -> Book {
        self.book
    }
}

fn map_book_chapters<F>(items: &mut [BookItem], handler: &F)
where
    F: Fn(&Chapter) -> String + Sync + Send,
{
    for (index, item) in items.iter_mut().enumerate() {
        let path = vec![index];
        map_book_item(item, path, handler);
    }
}

fn map_book_chapters_parallel<F>(items: &mut [BookItem], handler: &F)
where
    F: Fn(&Chapter) -> String + Sync + Send,
{
    items.par_iter_mut().enumerate().for_each(|(index, item)| {
        map_book_item(item, vec![index], handler);
    });
}

fn map_book_item<F>(item: &mut BookItem, path: Vec<usize>, handler: &F)
where
    F: Fn(&Chapter) -> String + Sync + Send,
{
    if let BookItem::Chapter(chapter) = item {
        let content = handler(&*chapter);
        chapter.content = content;

        for (index, item) in chapter.sub_items.iter_mut().enumerate() {
            let mut child_path = path.clone();
            child_path.push(index);
            map_book_item(item, child_path, handler);
        }
    }
}
