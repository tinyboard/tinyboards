DROP TRIGGER IF EXISTS wiki_page_initial_revision ON wiki_pages;
DROP FUNCTION IF EXISTS trg_wiki_page_initial_revision() CASCADE;

DROP TABLE IF EXISTS wiki_approved_contributors CASCADE;
DROP TABLE IF EXISTS wiki_page_revisions CASCADE;
DROP TABLE IF EXISTS wiki_pages CASCADE;
