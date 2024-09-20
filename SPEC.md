# (Incomplete) CLIP File Spec

## Header

see [head.rs](src/chunks/head.rs)

| Size | Type   | Description            |
|------|--------|------------------------|
| 8    | bytes  | File header "CSFCHUNK" |
| 8    | u64 BE | File size              |
| 8    | u64 BE | Head position = 24     |

| Size | Type   | Description                                                           |
|------|--------|-----------------------------------------------------------------------|
| 8    | bytes  | Header "CHNKHead"                                                     |
| 8    | u64 BE | Body size = 40                                                        |
| 8    | u64 BE | ??? = 256                                                             |
| 8    | u64 BE | [Sqlite chunk](#Sqlite-Chunk) position                                |
| 8    | u64 BE | ??? = 16                                                              |
| 16   | ???    | ??? Changes every save, <br/>file still opens when this is nulled out |

## Sqlite

see [sqli/mod.rs](src/chunks/sqli/mod.rs)

| Size | Type   | Description                             |
|------|--------|-----------------------------------------|
| 8    | bytes  | Header "CHNKSqli"                       |
| 8    | u64 BE | Sqlite data size                        |
| -    | -      | [Sqlite3](https://www.sqlite.org/) data |

The ExternalTableAndColumName contains a list of all the table names referring to the external IDs.

These external IDs are present in the ExternalChunk table as offsets of [external (Exta) chunks](#External-Chunk).

## External

see [exta/mod.rs](src/chunks/exta/mod.rs)

| Size | Type   | Description            |
|------|--------|------------------------|
| 8    | bytes  | Header "CHNKExta"      |
| 8    | u64 BE | chunk size             |
| 8    | u64 BE | ext ID length = 40     |
| 40   | bytes  | ext ID                 |
| 8    | u64 BE | chunk body size        |
| -    | -      | chunk body (see below) |

The external have several different types of data 
depending on which table(from Sqlite chunk's ExternalTableAndColumName) it was found in.

| Table                        | Description |
|------------------------------|-------------|
| [Offscreen](#Exta-Offscreen) | Raster data | 
| TODO                         |             |
|                              |             | 
|                              |             |

### Offscreen

See [exta/offscreen.rs](src/chunks/exta/offscreen.rs) for an outline
and [blockdecode.py](scripts/blockdecode.py) to export the data to images using Pillow.

Each block body contains a zipped 256x256 portion of the image. 
The unzipped data has a transparency mask (65536 bytes), followed by color data (rest of the file).
Note that the color data is in BGRA format.  
