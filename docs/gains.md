# Gains Calculation

This document describes the calculation of gains and the distribution of fees in different transaction scenarios.

---

## Forex Purchase (base-to-foreign)

In a foreign exchange purchase, we are opening a foreign asset position in the portfolio.
We are exchanging the base asset for a foreign asset.
Fees must not be booked as losses on the day of the purchase.
Instead, fees must be integrated into the stock valuation (fees increase the asset layer price).
The loss incurred by the fees will become effective once the asset is sold:
with constant proceeds and an increased stock valuation, gains decrease according to
`gains = proceeds - costs`, where `costs = base_value + fees`.

### Fees paid in Disposed Asset

Fees paid in the disposed asset increase the costs of the purchase.

> Example: purchase 2 ADA for 4 EUR, pay 0.10 EUR fees.
> * increase disposed amount by the fees: 4 EUR + 0.10 EUR = 4.10 EUR
> * leave received amount unadjusted at 1 ADA
> * record inflow of 2 ADA for 4.10 EUR
> * the effective asset price is 2.05 EUR/ADA

### Fees paid in Received Asset

Fees paid in the received asset decrease the actual amount of the received asset.
We record the inflow of the decreased amount and do not adjust the proceeds.
This results in an increased stock valuation.

> Example: purchase 2 ADA for 4 EUR, pay 0.05 ADA fees.
> * decrease received amount by the fee: 2 ADA - 0.05 ADA = 1.95 ADA
> * leave disposed amount unadjusted at 4 EUR
> * record inflow of 1.95 ADA for 4 EUR
> * the effective price becomes 2.05 EUR/ADA

### Fees paid in Third Asset

If the fees are paid in a third asset, 
we must first withdraw the third asset from the inventory and determine the FIFO/LIFO cost.

With the cost of the fee, expressed in base asset terms, we then continue as if the fee were paid in the base asset.
We increase the disposed amount by the fee value, and leave the received amount untouched.

> Example: purchase 2 ADA for 4 EUR, pay 0.001 BNB fees.
> * withdraw 0.001 BNB from BNB inventory at FIFO/LIFO price
> * in this example, 0.001 BNB = 0.10 EUR
> * notice that the withdrawal of BNB credits the base asset account with 0.10 EUR.
>   Because we do not actually receive this extra amount of money, we add it to the disposed amount to get rid of it:
> * increase disposed amount by the fee: 4 EUR + 0.10 EUR = 4.10 EUR
> * leave received amount unadjusted at 2 ADA
> * record inflow of 2 ADA for 4.10 EUR
> * the effective price becomes 2.05 EUR/ADA

---

## Forex Sale (foreign-to-base)

In a foreign exchange sale, we cash out on a foreign asset.
We are exchanging the foreign asset for the base asset.
Fees decrease the gains from the sale.

### Fees paid in Received Asset

Fees paid in the received asset decrease the actual amount of the received asset.
We record the inflow of the decreased amount and do not adjust the sale.
This results in decreased proceeds.

> Example: sell 2 ADA for 4 EUR, pay 0.10 EUR fees.
> * decrease received amount by the fee: 4 EUR - 0.10 EUR = 3.90 EUR
> * leave disposed amount unadjusted at 2 ADA
> * record outflow of 2 ADA for 3.90 EUR
> * the effective price becomes 1.95 EUR/ADA

### Fees paid in Disposed Asset

Fees paid in the disposed asset increase the disposed amount.
With constant proceeds, this results in a lower price.

> Example: sell 2 ADA for 4.00 EUR, pay 0.05 ADA fees.
> * increase disposed amount by the fee: 2 ADA + 0.05 ADA = 2.05 ADA
> * leave received amount unadjusted at 4 EUR
> * record outflow of 2.05 ADA for 4.00 EUR
> * the effective price becomes 1.95 EUR/ADA

### Fees paid in Third Asset

If the fee is paid in a third asset, 
we must first withdraw the asset from the inventory and determine the FIFO/LIFO cost.

With the cost of the fee, expressed in base asset terms, we then continue as if the fee was paid in the base asset.
We increase the disposed amount by the fee value, and leave the received amount untouched.

> Example: sell 2 ADA for 4 EUR, pay 0.001 BNB fees.
> * withdraw 0.001 BNB from BNB inventory at FIFO/LIFO price
> * in this example, 0.001 BNB = 0.10 EUR
> * notice that the withdrawal of BNB credits the base asset account with 0.10 EUR. 
>   Because we do not actually receive this amount of extra money, 
>   we adjust the fee so that `actual_proceeds = decrease_amount + fees`:
> * decrease received amount by the fee: 4 EUR - 0.10 EUR = 3.90 EUR
> * leave disposed amount unadjusted at 2 ADA
> * record outflow of 2 ADA for 3.90 EUR
> * the effective price becomes 1.95 EUR/ADA

---

## Foreign-to-Foreign Trade

This is the exchange of one foreign asset for another.

For proper gains calculations, we need to split the trade into two parts:
1) FOREX Sale
2) FOREX Purchase

In order to find the correct price for the FOREX sale, we query the current market price.

> Example: sell IOTA to buy ADA.
> 1) see IOTA for EUR (at market price)
> 2) use the received EUR to purchase ADA

Fees are split evenly between the two partial trades,
and purchase and sale rules apply recursively.

> Example with Fees: 
> 
> Purchase 3 IOTA for 2 ADA (market price: 2.00 EUR/ADA), 
> pay 0.001 BNB in fees
> 1) record sale of 2 ADA for 4.00 EUR, with 0.0005 BNB in fees
>   * withdraw 0.0005 BNB from inventory, at FIFO/LIFO price
>   * in this example, 0.0005 BNB = 0.05 EUR
>   * decrease received amount by the fee: 4 EUR - 0.05 EUR = 3.95 EUR
>   * leave disposed amount unadjusted at 2 ADA
>   * record outflow of 2 ADA for 3.95 EUR
>   * record a **loss of 0.05 EUR**
> 2) record purchase of 3 IOTA for 4.00 EUR, with 0.0005 BNB in fees
>   * withdraw 0.0005 BNB from inventory at FIFO/LIFO price
>   * in this example, 0.0005 BNB = 0.05 EUR
>   * increase disposed amount by the fee: 4 EUR + 0.05 EUR = 4.05 EUR
>   * leave received amount unadjusted at 2 ADA
>   * record inflow of 2 ADA for 4.05 EUR
>   * notice that the stock valuation is **0.05 EUR higher** than the net price. This will lead to a loss on sale.
>
> Analysis:
> * the realized loss from the sale is: 0.05 EUR
> * the unrealized loss (increased stock valuation) from the purchase is: 0.05 EUR
