# Commands
```
cargo run create-market --keypair dev_keypair.json && \
spl-token create-token t1_keypair.json && \
spl-token create-token t2_keypair.json && \
cargo run create-liquidity --market 3E1nNz4FiptsBW8vj36zQEikH4AYjhqSD3jfWkjc4LZV --token 8LwTcEgjkSUW2PMfoucBmUfRPtJEe5Q4JnJJKHmdmNAX && \
cargo run create-collateral --market 3E1nNz4FiptsBW8vj36zQEikH4AYjhqSD3jfWkjc4LZV --token 7EB8ikCxDwQuVP2kuqbyKaptieUZ2ptCkhY2tzZom7oR
```

```
cargo run update-liquidity --pubkey BrmEecfTGZFoygN4RVUvPC3wNeGoTx23sor8r9R12toX Active && \
cargo run update-collateral --pubkey A1EgEXQ4p3R6vgiv35gQNnc198QZ5D3YTL4edpmcnwQH Active
```