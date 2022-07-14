# CryptoTax

## Usage 

```
CryptoTax 0.1.0
Tim Hopp
Processes transaction statements into capital gains statements

USAGE:
    capital_tax.exe [FLAGS] [OPTIONS]

FLAGS:
        --clear      Clears the price cache
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --config <config_path>    Config file
    -i, --input <input_path>      Transaction file to process
    -o, --output <output_path>    Capital Gains Statement to write
```

## Example Files

### Configuration

An example configuration file for the CryptoTax generation. 
The file defines a base currency, an accounting method (FIFO), precision, and an API key for one API (coinapi).
See [config.yaml](./docs/example/config.yaml).
```
---
  base_asset: "EUR"
  method: "FIFO"
  currency_precision: 0.00000001
  api_key:
    coinapi: "73034021-THIS-IS-SAMPLE-KEY"
```

### Input file

A file that lists some real-world crypto transactions.
See [transactions.csv](./docs/example/transactions.csv).

### Output file

This file contains our transactions restructured into a format better suited for tax reporting.
The file also contains asset prices, losses and gains.
See [cashflows.csv](./docs/example/cashflows.csv).

### Price cache

To save API calls, the CryptoTax program stores prior price queries in a local cache file.
See [.price_cache](./docs/example/.price_cache).

## Run the Example

1. Replace the API key in `config.yaml` with a valid key.

2. Run the commands from the repo root directory:
```
cd ./docs/example
rm .price_cache cashflows.csv
cargo run --manifest-path ../../Cargo.toml -- -c config.yaml -i transactions.csv -o cashflows.csv
```

3. Observe the console output. See [output.txt](./docs/example/output.txt).
```
Processing record #1
Processing record #2
Processing record #3
--- running price query: BNB/EUR, 2021-03-18T12:43:12Z
--- success: price = 223.01685521866412
--- 98 API calls left
Processing record #4
--- running price query: USDT/EUR, 2021-03-18T16:23:30Z
--- success: price = 0.8377969550721581
--- 99 API calls left
Processing record #5
--- running price query: USDT/EUR, 2021-03-18T16:48:56Z
--- success: price = 0.8386905588345689
--- 97 API calls left

...
```

4. Inspect the output files.
