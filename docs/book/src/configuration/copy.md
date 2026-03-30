# [copy] / [copy-to] / [copy-from]

Default options for COPY TO and COPY FROM operations. Values here can be overridden with `WITH` clauses in the COPY command.

## [copy] — shared defaults

```ini
[copy]
numprocesses = 4
maxattempts = 5
reportfrequency = 0.25
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `numprocesses` | integer | `4` | Number of worker processes. |
| `maxattempts` | integer | `5` | Maximum retry attempts per batch. |
| `reportfrequency` | float | `0.25` | Progress report interval in seconds. |

## [copy-to] — export defaults

```ini
[copy-to]
pagesize = 1000
pagetimeout = 10
maxrequests = 6
maxoutputsize = -1
floatprecision = 5
doubleprecision = 12
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `pagesize` | integer | `1000` | Number of rows per page when reading. |
| `pagetimeout` | integer | `10` | Timeout for each page fetch in seconds. |
| `begintoken` | string | | Start token for range export. |
| `endtoken` | string | | End token for range export. |
| `maxrequests` | integer | `6` | Maximum concurrent page requests. |
| `maxoutputsize` | integer | `-1` | Maximum rows to export (`-1` = unlimited). |
| `floatprecision` | integer | `5` | Float decimal digits in CSV output. |
| `doubleprecision` | integer | `12` | Double decimal digits in CSV output. |

## [copy-from] — import defaults

```ini
[copy-from]
chunksize = 1000
ingestrate = 100000
maxbatchsize = 20
minbatchsize = 2
maxparseerrors = -1
maxinserterrors = -1
preparedstatements = true
ttl = -1
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `chunksize` | integer | `1000` | Number of rows per insert batch. |
| `ingestrate` | integer | `100000` | Target rows per second. |
| `maxbatchsize` | integer | `20` | Maximum rows per batch statement. |
| `minbatchsize` | integer | `2` | Minimum rows per batch statement. |
| `maxparseerrors` | integer | `-1` | Maximum parse errors before aborting (`-1` = unlimited). |
| `maxinserterrors` | integer | `-1` | Maximum insert errors before aborting (`-1` = unlimited). |
| `preparedstatements` | boolean | `true` | Use prepared statements for inserts. |
| `ttl` | integer | `-1` | TTL for inserted rows in seconds (`-1` = no TTL). |
