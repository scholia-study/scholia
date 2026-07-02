### Pipe db import from local machine to cluster

```sh
# terminal 1
just dev-forward
```

```sh
# terminal 2
just dev-run just db kant1
```

### Post Manual Checks

```sh
# German source (reviewed + modernized → struct)
cargo run -p md_prose_to_struct -- --corpus kant1

# English translation → struct
cargo run -p md_prose_to_struct -- --corpus kant1 --translation
```

Both use default asset paths:
- source mode: reads from assets/kant1/curated/md_modernized + assets/kant1/curated/md_reviewed, writes to assets/kant1/derived/output.json
- translation mode: reads from assets/kant1/curated/md_modernized_translated + assets/kant1/curated/md_modernized, writes to assets/kant1/derived/translation_output.json

### DB Import

```sh

cargo run -p struct_to_db -- --input-file assets/kant1/derived/output.json

cargo run -p struct_to_db -- --input-file assets/kant1/derived/translation_output.json --source-book-slug kritik-der-reinen-vernunft-b

```