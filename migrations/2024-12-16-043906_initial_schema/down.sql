-- This file should undo anything in `up.sql`
DROP TABLE sessions;
DROP TABLE users CASCADE;
DROP TABLE plans CASCADE;
DROP TABLE notifications;
DROP TABLE tags CASCADE;
DROP TABLE accounts CASCADE;
DROP TABLE currencies CASCADE;
DROP TABLE budgets CASCADE;
DROP TABLE transactions CASCADE;
DROP TABLE automations CASCADE;

DROP TABLE account_tags;
DROP TABLE transaction_tags;
