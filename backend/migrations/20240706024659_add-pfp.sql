-- Add migration script here
ALTER TABLE users
ADD avatar_url TEXT DEFAULT "https://cdn.discordapp.com/embed/avatars/1.png";