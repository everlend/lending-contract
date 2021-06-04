# Commands
```
cargo run create-market --keypair dev_keypair.json && \
spl-token create-token t1_keypair.json && \
spl-token create-token t2_keypair.json && \
cargo run create-liquidity-token --market 3E1nNz4FiptsBW8vj36zQEikH4AYjhqSD3jfWkjc4LZV --token 8LwTcEgjkSUW2PMfoucBmUfRPtJEe5Q4JnJJKHmdmNAX && \
cargo run create-collateral-token --market 3E1nNz4FiptsBW8vj36zQEikH4AYjhqSD3jfWkjc4LZV --token 7EB8ikCxDwQuVP2kuqbyKaptieUZ2ptCkhY2tzZom7oR
```