CREATE TABLE article_editorial_labels (
  article_id uuid NOT NULL,
  label_id uuid NOT NULL,
  applied_by uuid NOT NULL,
  applied_at timestamp with time zone NOT NULL,
  PRIMARY KEY (article_id, label_id),
  FOREIGN KEY (applied_by) REFERENCES users(id),
  FOREIGN KEY (article_id) REFERENCES articles(id) ON DELETE CASCADE,
  FOREIGN KEY (label_id) REFERENCES editorial_labels(id) ON DELETE CASCADE
);

CREATE TABLE article_quotations (
  id uuid NOT NULL,
  user_id uuid NOT NULL,
  article_id uuid,
  article_title text NOT NULL,
  author_display_name text NOT NULL,
  author_sort_name text,
  source_published_at timestamp with time zone,
  text text NOT NULL,
  html text NOT NULL,
  created_at timestamp with time zone NOT NULL,
  PRIMARY KEY (id),
  FOREIGN KEY (article_id) REFERENCES articles(id) ON DELETE SET NULL,
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE article_topics (
  article_id uuid NOT NULL,
  topic_id uuid NOT NULL,
  PRIMARY KEY (article_id, topic_id),
  FOREIGN KEY (article_id) REFERENCES articles(id) ON DELETE CASCADE,
  FOREIGN KEY (topic_id) REFERENCES topics(id) ON DELETE CASCADE
);

CREATE TABLE articles (
  id uuid NOT NULL,
  user_id uuid NOT NULL,
  title text NOT NULL,
  slug text NOT NULL,
  description text,
  markdown text NOT NULL,
  html text NOT NULL,
  status article_status NOT NULL,
  published_at timestamp with time zone,
  created_at timestamp with time zone NOT NULL,
  updated_at timestamp with time zone NOT NULL,
  PRIMARY KEY (id),
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE books (
  id uuid NOT NULL,
  source_id uuid NOT NULL,
  slug text NOT NULL,
  language text NOT NULL,
  about_text text,
  admin_notes text,
  created_at timestamp with time zone NOT NULL,
  updated_at timestamp with time zone NOT NULL,
  content_hash text,
  PRIMARY KEY (id),
  FOREIGN KEY (source_id) REFERENCES sources(id)
);

CREATE TABLE content_blocks (
  id uuid NOT NULL,
  book_id uuid NOT NULL,
  node_id uuid NOT NULL,
  position smallint NOT NULL,
  block_type block_type NOT NULL,
  paragraph_number integer,
  text text NOT NULL,
  html text NOT NULL,
  original_text text,
  original_html text,
  admin_notes text,
  created_at timestamp with time zone NOT NULL,
  updated_at timestamp with time zone NOT NULL,
  figure_number integer,
  PRIMARY KEY (id),
  FOREIGN KEY (book_id) REFERENCES books(id) ON DELETE CASCADE,
  FOREIGN KEY (node_id) REFERENCES toc_nodes(id) ON DELETE CASCADE
);

CREATE TABLE cross_references (
  id uuid NOT NULL,
  book_id uuid NOT NULL,
  label text NOT NULL,
  description text,
  source_node_id uuid NOT NULL,
  source_block_id uuid,
  source_sentence_start_id uuid,
  source_sentence_end_id uuid,
  target_node_id uuid NOT NULL,
  target_block_id uuid,
  target_sentence_start_id uuid,
  target_sentence_end_id uuid,
  metadata jsonb NOT NULL,
  admin_notes text,
  created_at timestamp with time zone NOT NULL,
  updated_at timestamp with time zone NOT NULL,
  PRIMARY KEY (id),
  FOREIGN KEY (book_id) REFERENCES books(id) ON DELETE CASCADE,
  FOREIGN KEY (source_block_id) REFERENCES content_blocks(id) ON DELETE CASCADE,
  FOREIGN KEY (source_node_id) REFERENCES toc_nodes(id) ON DELETE CASCADE,
  FOREIGN KEY (source_sentence_end_id) REFERENCES sentences(id) ON DELETE CASCADE,
  FOREIGN KEY (source_sentence_start_id) REFERENCES sentences(id) ON DELETE CASCADE,
  FOREIGN KEY (target_block_id) REFERENCES content_blocks(id) ON DELETE CASCADE,
  FOREIGN KEY (target_node_id) REFERENCES toc_nodes(id) ON DELETE CASCADE,
  FOREIGN KEY (target_sentence_end_id) REFERENCES sentences(id) ON DELETE CASCADE,
  FOREIGN KEY (target_sentence_start_id) REFERENCES sentences(id) ON DELETE CASCADE
);

CREATE TABLE cross_translation_alignments (
  book_id uuid NOT NULL,
  system_slug text NOT NULL,
  source_ref text NOT NULL,
  local_ref_value text NOT NULL,
  canonical_source_ref text,
  canonical_ref_value text,
  PRIMARY KEY (book_id, system_slug, source_ref, local_ref_value),
  FOREIGN KEY (book_id) REFERENCES books(id) ON DELETE CASCADE
);

CREATE TABLE editorial_labels (
  id uuid NOT NULL,
  name text NOT NULL,
  slug text NOT NULL,
  description text,
  revokes_on_edit boolean NOT NULL,
  sort_order integer NOT NULL,
  created_at timestamp with time zone NOT NULL,
  PRIMARY KEY (id)
);

CREATE TABLE email_verification_tokens (
  id uuid NOT NULL,
  user_id uuid NOT NULL,
  token_hash text NOT NULL,
  expires_at timestamp with time zone NOT NULL,
  used_at timestamp with time zone,
  created_at timestamp with time zone NOT NULL,
  PRIMARY KEY (id),
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE facsimile_pages (
  id uuid NOT NULL,
  reference_system_id uuid NOT NULL,
  ref_value text NOT NULL,
  storage_key text NOT NULL,
  caption text,
  admin_notes text,
  created_by uuid NOT NULL,
  created_at timestamp with time zone NOT NULL,
  updated_at timestamp with time zone NOT NULL,
  PRIMARY KEY (id),
  FOREIGN KEY (created_by) REFERENCES users(id),
  FOREIGN KEY (reference_system_id) REFERENCES reference_systems(id) ON DELETE CASCADE
);

CREATE TABLE feedback (
  id uuid NOT NULL,
  user_id uuid,
  body text NOT NULL,
  url text,
  user_agent text,
  viewport_w integer,
  viewport_h integer,
  status feedback_status NOT NULL,
  admin_notes text,
  handled_by uuid,
  created_at timestamp with time zone NOT NULL,
  updated_at timestamp with time zone NOT NULL,
  PRIMARY KEY (id),
  FOREIGN KEY (handled_by) REFERENCES users(id) ON DELETE SET NULL,
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
);

CREATE TABLE footnotes (
  id uuid NOT NULL,
  book_id uuid NOT NULL,
  number integer NOT NULL,
  anchor_sentence_id uuid NOT NULL,
  created_at timestamp with time zone NOT NULL,
  updated_at timestamp with time zone NOT NULL,
  PRIMARY KEY (id),
  FOREIGN KEY (anchor_sentence_id) REFERENCES sentences(id) ON DELETE CASCADE,
  FOREIGN KEY (book_id) REFERENCES books(id) ON DELETE CASCADE
);

CREATE TABLE page_markers (
  id uuid NOT NULL,
  system_id uuid NOT NULL,
  sentence_id uuid NOT NULL,
  ref_value text NOT NULL,
  sort_order integer NOT NULL,
  char_offset integer,
  admin_notes text,
  created_at timestamp with time zone NOT NULL,
  updated_at timestamp with time zone NOT NULL,
  PRIMARY KEY (id),
  FOREIGN KEY (sentence_id) REFERENCES sentences(id) ON DELETE CASCADE,
  FOREIGN KEY (system_id) REFERENCES reference_systems(id) ON DELETE CASCADE
);

CREATE TABLE password_reset_tokens (
  id uuid NOT NULL,
  user_id uuid NOT NULL,
  token_hash text NOT NULL,
  expires_at timestamp with time zone NOT NULL,
  used_at timestamp with time zone,
  created_at timestamp with time zone NOT NULL,
  PRIMARY KEY (id),
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE persons (
  id uuid NOT NULL,
  name text NOT NULL,
  sort_name text,
  protected boolean NOT NULL,
  created_by uuid NOT NULL,
  created_at timestamp with time zone NOT NULL,
  PRIMARY KEY (id),
  FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE TABLE quotation_note_tags (
  note_id uuid NOT NULL,
  tag_id uuid NOT NULL,
  PRIMARY KEY (note_id, tag_id),
  FOREIGN KEY (note_id) REFERENCES quotation_notes(id) ON DELETE CASCADE,
  FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE TABLE quotation_notes (
  id uuid NOT NULL,
  quotation_id uuid,
  article_quotation_id uuid,
  body text NOT NULL,
  created_at timestamp with time zone NOT NULL,
  updated_at timestamp with time zone NOT NULL,
  PRIMARY KEY (id),
  FOREIGN KEY (article_quotation_id) REFERENCES article_quotations(id) ON DELETE CASCADE,
  FOREIGN KEY (quotation_id) REFERENCES quotations(id) ON DELETE CASCADE
);

CREATE TABLE quotations (
  id uuid NOT NULL,
  user_id uuid NOT NULL,
  book_id uuid NOT NULL,
  anchor_node_id uuid NOT NULL,
  anchor_sentence_start_id uuid NOT NULL,
  anchor_sentence_end_id uuid,
  sentence_kind sentence_kind NOT NULL,
  source_id uuid NOT NULL,
  created_at timestamp with time zone NOT NULL,
  PRIMARY KEY (id),
  FOREIGN KEY (anchor_node_id) REFERENCES toc_nodes(id) ON DELETE CASCADE,
  FOREIGN KEY (anchor_sentence_end_id) REFERENCES sentences(id) ON DELETE CASCADE,
  FOREIGN KEY (anchor_sentence_start_id) REFERENCES sentences(id) ON DELETE CASCADE,
  FOREIGN KEY (book_id) REFERENCES books(id) ON DELETE CASCADE,
  FOREIGN KEY (source_id) REFERENCES sources(id),
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE reference_systems (
  id uuid NOT NULL,
  book_id uuid NOT NULL,
  slug text NOT NULL,
  label text NOT NULL,
  description text,
  ref_type text NOT NULL,
  admin_notes text,
  created_at timestamp with time zone NOT NULL,
  updated_at timestamp with time zone NOT NULL,
  PRIMARY KEY (id),
  FOREIGN KEY (book_id) REFERENCES books(id) ON DELETE CASCADE
);

CREATE TABLE released_handles (
  handle text NOT NULL,
  user_id uuid NOT NULL,
  released_at timestamp with time zone NOT NULL,
  PRIMARY KEY (handle),
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE resources (
  id uuid NOT NULL,
  book_id uuid NOT NULL,
  resource_type resource_type NOT NULL,
  anchor_node_id uuid NOT NULL,
  anchor_block_id uuid,
  anchor_sentence_start_id uuid,
  anchor_sentence_end_id uuid,
  sentence_kind sentence_kind NOT NULL,
  source_id uuid,
  source_page_start integer,
  source_page_end integer,
  source_location_freeform text,
  verbatim_kind verbatim_kind,
  quoted_text text,
  editor_note text,
  is_featured boolean NOT NULL,
  archived_at timestamp with time zone,
  archived_by uuid,
  admin_notes text,
  created_at timestamp with time zone NOT NULL,
  updated_at timestamp with time zone NOT NULL,
  PRIMARY KEY (id),
  FOREIGN KEY (anchor_block_id) REFERENCES content_blocks(id) ON DELETE CASCADE,
  FOREIGN KEY (anchor_node_id) REFERENCES toc_nodes(id) ON DELETE CASCADE,
  FOREIGN KEY (anchor_sentence_end_id) REFERENCES sentences(id) ON DELETE CASCADE,
  FOREIGN KEY (anchor_sentence_start_id) REFERENCES sentences(id) ON DELETE CASCADE,
  FOREIGN KEY (archived_by) REFERENCES users(id),
  FOREIGN KEY (book_id) REFERENCES books(id) ON DELETE CASCADE,
  FOREIGN KEY (source_id) REFERENCES sources(id)
);

CREATE TABLE roles (
  id uuid NOT NULL,
  name text NOT NULL,
  created_at timestamp with time zone NOT NULL,
  updated_at timestamp with time zone NOT NULL,
  PRIMARY KEY (id)
);

CREATE TABLE sentences (
  id uuid NOT NULL,
  book_id uuid NOT NULL,
  node_id uuid NOT NULL,
  block_id uuid,
  footnote_id uuid,
  position smallint NOT NULL,
  sentence_number integer,
  source_sentence_start_id uuid,
  source_sentence_end_id uuid,
  text text NOT NULL,
  html text NOT NULL,
  original_text text,
  original_html text,
  admin_notes text,
  created_at timestamp with time zone NOT NULL,
  updated_at timestamp with time zone NOT NULL,
  segment smallint,
  natural_key text,
  PRIMARY KEY (id),
  FOREIGN KEY (footnote_id) REFERENCES footnotes(id) ON DELETE CASCADE,
  FOREIGN KEY (block_id) REFERENCES content_blocks(id) ON DELETE CASCADE,
  FOREIGN KEY (book_id) REFERENCES books(id) ON DELETE CASCADE,
  FOREIGN KEY (node_id) REFERENCES toc_nodes(id) ON DELETE CASCADE,
  FOREIGN KEY (source_sentence_end_id) REFERENCES sentences(id) ON DELETE SET NULL,
  FOREIGN KEY (source_sentence_start_id) REFERENCES sentences(id) ON DELETE SET NULL
);

CREATE TABLE source_persons (
  source_id uuid NOT NULL,
  person_id uuid NOT NULL,
  role source_person_role NOT NULL,
  position smallint NOT NULL,
  PRIMARY KEY (source_id, person_id, role),
  FOREIGN KEY (person_id) REFERENCES persons(id) ON DELETE RESTRICT,
  FOREIGN KEY (source_id) REFERENCES sources(id) ON DELETE CASCADE
);

CREATE TABLE sources (
  id uuid NOT NULL,
  source_type source_type NOT NULL,
  title text NOT NULL,
  title_display text,
  publication_year smallint,
  publisher text,
  isbn text[],
  doi text,
  edition text,
  volume text,
  journal_name text,
  url text,
  page_start integer,
  page_end integer,
  parent_source_id uuid,
  translation_of_id uuid,
  protected boolean NOT NULL,
  created_by uuid NOT NULL,
  created_at timestamp with time zone NOT NULL,
  PRIMARY KEY (id),
  FOREIGN KEY (created_by) REFERENCES users(id),
  FOREIGN KEY (parent_source_id) REFERENCES sources(id) ON DELETE SET NULL,
  FOREIGN KEY (translation_of_id) REFERENCES sources(id) ON DELETE SET NULL
);

CREATE TABLE stripe_processed_events (
  event_id text NOT NULL,
  processed_at timestamp with time zone NOT NULL,
  PRIMARY KEY (event_id)
);

CREATE TABLE subscriptions (
  id uuid NOT NULL,
  user_id uuid NOT NULL,
  stripe_subscription_id text NOT NULL,
  stripe_price_id text NOT NULL,
  tier text NOT NULL,
  status text NOT NULL,
  current_period_end timestamp with time zone NOT NULL,
  cancel_at_period_end boolean NOT NULL,
  created_at timestamp with time zone NOT NULL,
  updated_at timestamp with time zone NOT NULL,
  PRIMARY KEY (id),
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE tags (
  id uuid NOT NULL,
  user_id uuid NOT NULL,
  name text NOT NULL,
  PRIMARY KEY (id),
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE toc_nodes (
  id uuid NOT NULL,
  book_id uuid NOT NULL,
  parent_id uuid,
  source_node_id uuid,
  source_id uuid,
  source_ref text NOT NULL,
  slug text NOT NULL,
  path ltree NOT NULL,
  sort_order integer NOT NULL,
  depth smallint NOT NULL,
  label text NOT NULL,
  label_html text,
  admin_notes text,
  created_at timestamp with time zone NOT NULL,
  updated_at timestamp with time zone NOT NULL,
  content_hash text,
  PRIMARY KEY (id),
  FOREIGN KEY (book_id) REFERENCES books(id) ON DELETE CASCADE,
  FOREIGN KEY (parent_id) REFERENCES toc_nodes(id) ON DELETE CASCADE,
  FOREIGN KEY (source_id) REFERENCES sources(id) ON DELETE SET NULL,
  FOREIGN KEY (source_node_id) REFERENCES toc_nodes(id) ON DELETE SET NULL
);

CREATE TABLE topics (
  id uuid NOT NULL,
  name text NOT NULL,
  slug text NOT NULL,
  created_at timestamp with time zone NOT NULL,
  updated_at timestamp with time zone NOT NULL,
  PRIMARY KEY (id)
);

CREATE TABLE user_oauth_accounts (
  id uuid NOT NULL,
  user_id uuid NOT NULL,
  provider text NOT NULL,
  provider_user_id text NOT NULL,
  email text,
  created_at timestamp with time zone NOT NULL,
  updated_at timestamp with time zone NOT NULL,
  PRIMARY KEY (id),
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE user_roles (
  user_id uuid NOT NULL,
  role_id uuid NOT NULL,
  PRIMARY KEY (user_id, role_id),
  FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE,
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE user_sessions (
  session_id text NOT NULL,
  user_id uuid NOT NULL,
  created_at timestamp with time zone NOT NULL,
  PRIMARY KEY (session_id),
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE users (
  id uuid NOT NULL,
  display_name text NOT NULL,
  sort_name text,
  handle text,
  handle_changed_at timestamp with time zone,
  bio text,
  title text,
  location text,
  website_url text,
  email text NOT NULL,
  password_hash text,
  avatar_url text,
  email_verified_at timestamp with time zone,
  sessions_invalidated_at timestamp with time zone,
  stripe_customer_id text,
  admin_notes text,
  created_at timestamp with time zone NOT NULL,
  updated_at timestamp with time zone NOT NULL,
  PRIMARY KEY (id)
);

