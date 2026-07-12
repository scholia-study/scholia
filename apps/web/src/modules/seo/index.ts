export type { BookMeta, BookTranslationRef } from "./bookMeta";
export { toBookMeta } from "./bookMeta";
export { SEO_COPY, SITE_NAME, siteTitle } from "./copy";
export type { OgType, SeoHeadArgs } from "./head";
export { metaDescription, seoHead, stripHtml } from "./head";
export type { ArticleJsonLdMeta, ProfileJsonLdMeta } from "./jsonld";
export {
    articleJsonLd,
    bookJsonLd,
    breadcrumbJsonLd,
    chapterJsonLd,
    profileJsonLd,
} from "./jsonld";
export { findTocTrail } from "./toc";
