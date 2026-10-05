-- Dusk Supabase Storage setup
-- Run this in the Dusk Supabase project SQL editor or apply it through the Supabase connector.
-- The bucket is private. Files are isolated by the authenticated Supabase user UUID
-- in the first path segment: <auth.uid()>/profiles/<profile>/games/<game>/backups/<backup>/...

insert into storage.buckets (id, name, public)
values ('dusk-savefiles', 'dusk-savefiles', false)
on conflict (id) do update set public = false;

drop policy if exists "dusk_savefiles_insert_own" on storage.objects;
create policy "dusk_savefiles_insert_own"
on storage.objects
for insert
to authenticated
with check (
  bucket_id = 'dusk-savefiles'
  and (storage.foldername(name))[1] = (select auth.uid()::text)
);

drop policy if exists "dusk_savefiles_select_own" on storage.objects;
create policy "dusk_savefiles_select_own"
on storage.objects
for select
to authenticated
using (
  bucket_id = 'dusk-savefiles'
  and owner_id = (select auth.uid()::text)
);

drop policy if exists "dusk_savefiles_update_own" on storage.objects;
create policy "dusk_savefiles_update_own"
on storage.objects
for update
to authenticated
using (
  bucket_id = 'dusk-savefiles'
  and owner_id = (select auth.uid()::text)
)
with check (
  bucket_id = 'dusk-savefiles'
  and (storage.foldername(name))[1] = (select auth.uid()::text)
);

drop policy if exists "dusk_savefiles_delete_own" on storage.objects;
create policy "dusk_savefiles_delete_own"
on storage.objects
for delete
to authenticated
using (
  bucket_id = 'dusk-savefiles'
  and owner_id = (select auth.uid()::text)
);

