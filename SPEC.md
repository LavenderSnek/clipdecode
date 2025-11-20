# (Incomplete) CLIP File Spec

## Chunks

see [chunk.rs](src/chunk.rs)

Sections of the clip file:
- CSFCHUNK: Start of file, contains file size and CHNKHead position
- CHNKHead: Contains CHNKSQLi position
- CHNKExta: Various types of data with mappings in the embedded sqlite DB. See [Exta](#Exta)
- CHNKSQLi: The body is a sqlite DB. Contains all metadata

## Exta

The `ExternalTableAndColumnName` SQLite table contians a list of tables and column names where external chunk IDs are found. A list of offsets for the chunks are found in the `ExternalChunk` table.

The following is an incomplete list of chunk types

### Offscreen

See [offscreen.rs](src/exta/offscreen.rs)

See [blockdecode.py](scripts/blockdecode.py)

Raster image data compressed with zlib. More info needed on `Offscreen.Attribute`.

### VectorObjectList

See [vector.rs](src/exta/vector.rs)
