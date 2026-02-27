# 03 - Finance Editing

## Overview

The Finance section allows viewing and editing the financial state of the player's farm. This is one of the most requested features for a save editor.

## Simple Mode

### Available Money
- Display of the farm's current balance
- Direct editing of the amount via a numeric field
- Validation: the amount must be zero or positive
- Automatic formatting with thousands separator based on locale

### Current Loan
- Display of the current loan amount
- Editing the loan amount
- Option to set the loan to zero (full repayment)
- Validation: the amount must be zero or positive

### Quick Actions
- **"Repay Loan"**: sets the loan to 0 and deducts the amount from available money (if sufficient), or simply sets the loan to 0 (user's choice)
- **"Add Money"**: preset buttons (+10,000, +100,000, +1,000,000) in addition to a free-form field

## Advanced Mode

### Farm Financial Statistics
View and edit cumulative counters:
- Distance traveled
- Fuel consumption
- Seed consumption
- Crop protection product consumption
- Hectares worked, cultivated, sown, sprayed, harvested, plowed
- Number of bales produced
- Number of missions completed

### Daily Financial History
View the financial history day by day:
- Vehicle costs
- Land purchases
- Production costs and revenue
- Property maintenance and revenue
- Mission revenue
- Wages paid
- Loan interest
- Miscellaneous income and expenses

> Note: editing the financial history is possible but not recommended (visual indicator). This data is informational and does not affect gameplay.

## Consistency Rules

- The money value is synchronized between `careerSavegame.xml` (`money` attribute) and `farms.xml` (`money` attribute of the farm)
- Any modification to money or loan in the interface updates both files simultaneously
