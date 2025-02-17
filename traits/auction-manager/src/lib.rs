// This pallet use The Open Runtime Module Library (ORML) which is a community maintained collection
// of Substrate runtime modules. Thanks to all contributors of orml.
// Ref: https://github.com/open-web3-stack/open-runtime-module-library
#![cfg_attr(not(feature = "std"), no_std)]

use codec::FullCodec;
use codec::{Decode, Encode};
use frame_support::dispatch::DispatchResult;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{traits::AtLeast32BitUnsigned, DispatchError, Perbill, RuntimeDebug};
use sp_std::{
	cmp::{Eq, PartialEq},
	fmt::Debug,
	vec::Vec,
};

use primitives::{AssetId, AuctionId, ClassId, FungibleTokenId, ItemId, MetaverseId, TokenId};

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum Change<Value> {
	/// No change.
	NoChange,
	/// Changed to new value.
	NewValue(Value),
}

#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum AuctionType {
	Auction,
	BuyNow,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ListingLevel<AccountId> {
	// Accepted bidders
	NetworkSpot(Vec<AccountId>),
	Global,
	Local(MetaverseId),
}

#[cfg_attr(feature = "std", derive(PartialEq, Eq))]
#[derive(Encode, Decode, Clone, RuntimeDebug, TypeInfo)]
pub struct AuctionItem<AccountId, BlockNumber, Balance> {
	pub item_id: ItemId<Balance>,
	pub recipient: AccountId,
	pub initial_amount: Balance,
	/// Current amount for sale
	pub amount: Balance,
	/// Auction start time
	pub start_time: BlockNumber,
	pub end_time: BlockNumber,
	pub auction_type: AuctionType,
	pub listing_level: ListingLevel<AccountId>,
	pub currency_id: FungibleTokenId,
	pub listing_fee: Perbill,
}

#[cfg_attr(feature = "std", derive(PartialEq, Eq))]
#[derive(Encode, Decode, Clone, RuntimeDebug, TypeInfo)]
pub struct AuctionItemV1<AccountId, BlockNumber, Balance> {
	pub item_id: ItemId<Balance>,
	pub recipient: AccountId,
	pub initial_amount: Balance,
	/// Current amount for sale
	pub amount: Balance,
	/// Auction start time
	pub start_time: BlockNumber,
	pub end_time: BlockNumber,
	pub auction_type: AuctionType,
	pub listing_level: ListingLevel<AccountId>,
	pub currency_id: FungibleTokenId,
}

/// Auction info.
#[cfg_attr(feature = "std", derive(PartialEq, Eq))]
#[derive(Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct AuctionInfo<AccountId, Balance, BlockNumber> {
	/// Current bidder and bid price.
	pub bid: Option<(AccountId, Balance)>,
	/// Define which block this auction will be started.
	pub start: BlockNumber,
	/// Define which block this auction will be ended.
	pub end: Option<BlockNumber>,
}

/// Abstraction over a simple auction system.
pub trait Auction<AccountId, BlockNumber> {
	/// The price to bid.
	type Balance: AtLeast32BitUnsigned + FullCodec + Copy + Default + Debug;

	/// The auction info of `id`
	fn auction_info(id: AuctionId) -> Option<AuctionInfo<AccountId, Self::Balance, BlockNumber>>;

	/// The auction item of `id`
	fn auction_item(id: AuctionId) -> Option<AuctionItem<AccountId, BlockNumber, Self::Balance>>;

	/// Update the auction info of `id` with `info`
	fn update_auction(id: AuctionId, info: AuctionInfo<AccountId, Self::Balance, BlockNumber>) -> DispatchResult;

	/// Update auction item of `id` with new `item_id`
	fn update_auction_item(id: AuctionId, item_id: ItemId<Self::Balance>) -> DispatchResult;

	/// Create new auction with specific startblock and endblock, return the id
	/// of the auction
	fn new_auction(
		recipient: AccountId,
		initial_amount: Self::Balance,
		start: BlockNumber,
		end: Option<BlockNumber>,
	) -> Result<AuctionId, DispatchError>;

	fn create_auction(
		auction_type: AuctionType,
		item_id: ItemId<Self::Balance>,
		end: Option<BlockNumber>,
		recipient: AccountId,
		initial_amount: Self::Balance,
		start: BlockNumber,
		listing_level: ListingLevel<AccountId>,
		listing_fee: Perbill,
	) -> Result<AuctionId, DispatchError>;

	/// Remove auction by `id`
	fn remove_auction(id: AuctionId, item_id: ItemId<Self::Balance>);

	fn auction_bid_handler(from: AccountId, id: AuctionId, value: Self::Balance) -> DispatchResult;

	fn buy_now_handler(from: AccountId, auction_id: AuctionId, value: Self::Balance) -> DispatchResult;

	fn local_auction_bid_handler(
		_now: BlockNumber,
		id: AuctionId,
		new_bid: (AccountId, Self::Balance),
		last_bid: Option<(AccountId, Self::Balance)>,
		social_currency_id: FungibleTokenId,
	) -> DispatchResult;

	fn collect_royalty_fee(
		high_bid_price: &Self::Balance,
		high_bidder: &AccountId,
		asset_id: &(ClassId, TokenId),
		social_currency_id: FungibleTokenId,
	) -> DispatchResult;
}

pub trait CheckAuctionItemHandler<Balance> {
	fn check_item_in_auction(item_id: ItemId<Balance>) -> bool;
}

/// The result of bid handling.
pub struct OnNewBidResult<BlockNumber> {
	/// Indicates if the bid was accepted
	pub accept_bid: bool,
	/// The auction end change.
	pub auction_end_change: Change<Option<BlockNumber>>,
}

/// Hooks for auction to handle bids.
pub trait AuctionHandler<AccountId, Balance, BlockNumber, AuctionId> {
	/// Called when new bid is received.
	/// The return value determines if the bid should be accepted and update
	/// auction end time. Implementation should reserve money from current
	/// winner and refund previous winner.
	fn on_new_bid(
		now: BlockNumber,
		id: AuctionId,
		new_bid: (AccountId, Balance),
		last_bid: Option<(AccountId, Balance)>,
	) -> OnNewBidResult<BlockNumber>;
	/// End an auction with `winner`
	fn on_auction_ended(id: AuctionId, winner: Option<(AccountId, Balance)>);
}

/// Swap manager to use swap functionality in different traits
pub trait SwapManager<AccountId, CurrencyId, Balance> {
	fn add_liquidity(
		who: &AccountId,
		token_id_a: FungibleTokenId,
		token_id_b: FungibleTokenId,
		max_amount_a: Balance,
		max_amount_b: Balance,
	) -> DispatchResult;
}
