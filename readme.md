# sainte_lague

A rust implementation of the **[Sainte-Laguë](https://en.wikipedia.org/wiki/Webster/Sainte-Lagu%C3%AB_method)** (also known as **Webster** or **Schepers**) method. Parliament seat allocation algorithm used in multiple countries such as Germany, Latvia, New Zealand etc…

*Attention: Since some countries (like Latvia or Norway) use a modification of the algorithm instead of this vanilla version, you should check your country's electoral legislature. Furthermore, I don't take any responsibility for the accuracy of the calculated numbers, even though I'm pretty confident with my implementation.*

[![Crate Version](https://img.shields.io/crates/v/sainte_lague.svg)](https://crates.io/crates/sainte_lague)
[![License](https://img.shields.io/github/license/juliuste/sainte_lague.rs.svg?style=flat)](license)

## Example

```rust
use sainte_lague::distribute;

// …
#[test]
fn german_bundestag_2013() {
	let votes = [41.5, 25.7, 8.6, 8.4];
	let seats = 631;

	let distribution = distribute(&votes, &seats, &false);
	let parliament = vec![311, 193, 64, 63];
	assert_eq!(distribution, Ok(parliament));
}
// …
```

**Full documentation on [docs.rs](https://docs.rs/sainte_lague/).**

## Similar projects

- [`largest-remainder-method`](https://crates.io/crates/largest-remainder-method) - A rust implementation of the Hare-Niemeyer / Hamilton / largest remainder method

## Contributing

If you found a bug or want to propose a feature, feel free to visit [the issues page](https://github.com/juliuste/sainte_lague.rs/issues).
