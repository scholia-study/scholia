import type { BookDetail } from "../../api/model";

export interface BookTranslationRef {
    slug: string;
    title: string;
    language: string;
}

/**
 * The scalar subset of BookDetail that route loaders return for
 * `head()`. Kept minimal on purpose: loaderData is dehydrated into the
 * SSR HTML separately from the react-query cache, so returning the full
 * BookDetail would serialize it twice.
 */
export interface BookMeta {
    title: string;
    author: string;
    language: string;
    publicationYear: number | null;
    publisher: string | null;
    /** Books translated FROM this one (Kant DE → [Kant EN]). */
    translations: BookTranslationRef[];
    /**
     * The hosted source-language book this one translates, if any.
     * `sibling_translations` are deliberately NOT carried: KJV is not a
     * translation *of* WEB, so linking siblings as workTranslation
     * would be semantically wrong.
     */
    sourceBookSlug: string | null;
}

export function toBookMeta(book: BookDetail): BookMeta {
    return {
        title: book.title,
        author: book.author,
        language: book.language,
        publicationYear: book.publication_year ?? null,
        publisher: book.publisher ?? null,
        translations: book.translations.map((t) => ({
            slug: t.slug,
            title: t.title,
            language: t.language,
        })),
        sourceBookSlug: book.source_book_slug ?? null,
    };
}
