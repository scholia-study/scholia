### Pipe db import from local machine to cluster

```sh
# terminal 1
pnpm db:dev:forward
```

```sh
# terminal 2
pnpm db:dev:run pnpm db:kant1
```

### Post Manual Checks

```sh
# German source (reviewed + modernized → struct)
cargo run -p kant1_md_to_struct

# English translation → struct
cargo run -p kant1_md_translation_to_struct
```

Both use default asset paths:
- kant1_md_to_struct: reads from assets/kant1/curated/md_modernized + assets/kant1/curated/md_reviewed, writes to assets/kant1/derived/md_to_struct/output.json
- kant1_md_translation_to_struct: reads from assets/kant1/curated/md_modernized_translated + assets/kant1/curated/md_modernized, writes to assets/kant1/derived/md_translation_to_struct/output.json

### DB Import

```sh

cargo run -p kant1_struct_to_db -- --input-file assets/kant1/derived/md_to_struct/output.json

cargo run -p kant1_struct_to_db -- --input-file assets/kant1/derived/md_translation_to_struct/output.json --source-book-slug kritik-der-reinen-vernunft-b

```