-- Registration no longer reveals whether an email is taken: attempts on
-- an existing address get the same response as a fresh signup, and the
-- account holder is notified by email instead. This timestamp throttles
-- that notice to once per cooldown window per address, so repeated
-- registration attempts cannot be used to bombard a victim's inbox.

ALTER TABLE users ADD COLUMN account_notice_last_sent_at TIMESTAMPTZ;
