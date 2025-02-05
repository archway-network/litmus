package testenv

import (
	"encoding/json"
	"fmt"
	"strings"
	"time"

	// helpers

	// tendermint
	"cosmossdk.io/errors"
	"cosmossdk.io/log"
	abci "github.com/cometbft/cometbft/abci/types"
	dbm "github.com/cosmos/cosmos-db"

	// cosmos-sdk
	cmtproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/server"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	banktestutil "github.com/cosmos/cosmos-sdk/x/bank/testutil"
	distributiontypes "github.com/cosmos/cosmos-sdk/x/distribution/types"
	minttypes "github.com/cosmos/cosmos-sdk/x/mint/types"
	slashingtypes "github.com/cosmos/cosmos-sdk/x/slashing/types"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"

	// wasmd

	wasmdKeeper "github.com/CosmWasm/wasmd/x/wasm/keeper"
	wasmtypes "github.com/CosmWasm/wasmd/x/wasm/types"

	// archway
	"github.com/archway-network/archway/app"
	rewards "github.com/archway-network/archway/x/rewards/types"

	sdkmath "cosmossdk.io/math"

	tmtypes "github.com/cometbft/cometbft/types"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"

	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"

	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"

	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
)

func GenesisStateWithValSet(appInstance *app.ArchwayApp) (app.GenesisState, secp256k1.PrivKey) {
	privVal := NewPV()
	pubKey, _ := privVal.GetPubKey()
	validator := tmtypes.NewValidator(pubKey, 1)
	valSet := tmtypes.NewValidatorSet([]*tmtypes.Validator{validator})

	// generate genesis account
	senderPrivKey := secp256k1.GenPrivKey()
	senderPrivKey.PubKey().Address()
	acc := authtypes.NewBaseAccountWithAddress(senderPrivKey.PubKey().Address().Bytes())

	//////////////////////
	balances := []banktypes.Balance{}
	genesisState := app.NewDefaultGenesisState(app.MakeEncodingConfig().Marshaler)
	genAccs := []authtypes.GenesisAccount{acc}
	authGenesis := authtypes.NewGenesisState(authtypes.DefaultParams(), genAccs)
	genesisState[authtypes.ModuleName] = appInstance.AppCodec().MustMarshalJSON(authGenesis)

	validators := make([]stakingtypes.Validator, 0, len(valSet.Validators))
	delegations := make([]stakingtypes.Delegation, 0, len(valSet.Validators))

	bondAmt := sdk.DefaultPowerReduction
	initValPowers := []abci.ValidatorUpdate{}

	for _, val := range valSet.Validators {
		pk, _ := cryptocodec.FromTmPubKeyInterface(val.PubKey)
		pkAny, _ := codectypes.NewAnyWithValue(pk)
		validator := stakingtypes.Validator{
			OperatorAddress: sdk.ValAddress(val.Address).String(),
			ConsensusPubkey: pkAny,
			Jailed:          false,
			Status:          stakingtypes.Bonded,
			Tokens:          bondAmt,
			DelegatorShares: sdkmath.LegacyOneDec(),
			Description:     stakingtypes.Description{},
			UnbondingHeight: int64(0),
			UnbondingTime:   time.Unix(0, 0).UTC(),
			Commission: stakingtypes.NewCommission(sdkmath.LegacyNewDecWithPrec(5, 2), // 5% rate
				sdkmath.LegacyNewDecWithPrec(20, 2), // 20% max rate
				sdkmath.LegacyNewDecWithPrec(1, 2),  // 1% max change rate
			),
			MinSelfDelegation: sdkmath.ZeroInt(),
		}
		validators = append(validators, validator)
		delegations = append(delegations, stakingtypes.NewDelegation(genAccs[0].GetAddress().String(), sdk.ValAddress(val.Address).String(), sdkmath.LegacyOneDec()))

		// add initial validator powers so consumer InitGenesis runs correctly
		pub, _ := val.ToProto()
		initValPowers = append(initValPowers, abci.ValidatorUpdate{
			Power:  val.VotingPower,
			PubKey: pub.PubKey,
		})
	}
	// set validators and delegations
	stakingGenesis := stakingtypes.NewGenesisState(stakingtypes.DefaultParams(), validators, delegations)
	genesisState[stakingtypes.ModuleName] = appInstance.AppCodec().MustMarshalJSON(stakingGenesis)

	// Set rewards
	mintGenesis := minttypes.NewGenesisState(minttypes.DefaultInitialMinter(), minttypes.DefaultParams())
	mintGenesis.Params.MintDenom = "aarch"
	mintGenesis.Params.InflationMin = sdkmath.LegacyNewDecWithPrec(7, 2)         // 7%
	mintGenesis.Params.InflationMax = sdkmath.LegacyNewDecWithPrec(20, 2)        // 20%
	mintGenesis.Params.InflationRateChange = sdkmath.LegacyNewDecWithPrec(13, 2) // 13%
	mintGenesis.Params.GoalBonded = sdkmath.LegacyNewDecWithPrec(67, 2)          // 67%
	mintGenesis.Minter.Inflation = sdkmath.LegacyNewDecWithPrec(13, 2)           // 13%
	mintGenesis.Minter.AnnualProvisions = sdkmath.LegacyNewDec(0)
	genesisState[minttypes.ModuleName] = appInstance.AppCodec().MustMarshalJSON(mintGenesis)

	// Set distribution just in case
	distributionGenesis := distributiontypes.DefaultGenesisState()
	distributionGenesis.Params = distributiontypes.Params{
		CommunityTax:        sdkmath.LegacyNewDecWithPrec(2, 2), // 2%
		BaseProposerReward:  sdkmath.LegacyNewDecWithPrec(1, 2), // 1%
		BonusProposerReward: sdkmath.LegacyNewDecWithPrec(4, 2), // 4%
		WithdrawAddrEnabled: true,
	}
	genesisState[distributiontypes.ModuleName] = appInstance.AppCodec().MustMarshalJSON(distributionGenesis)

	totalSupply := sdk.NewCoins()
	for _, b := range balances {
		// add genesis acc tokens to total supply
		totalSupply = totalSupply.Add(b.Coins...)
	}

	for range delegations {
		// add delegated tokens to total supply
		totalSupply = totalSupply.Add(sdk.NewCoin(sdk.DefaultBondDenom, bondAmt))
	}

	// add bonded amount to bonded pool module account
	balances = append(balances, banktypes.Balance{
		Address: authtypes.NewModuleAddress(stakingtypes.BondedPoolName).String(),
		Coins:   sdk.Coins{sdk.NewCoin(sdk.DefaultBondDenom, bondAmt)},
	})

	// update total supply
	bankGenesis := banktypes.NewGenesisState(
		banktypes.DefaultGenesisState().Params,
		balances,
		totalSupply,
		[]banktypes.Metadata{},
		[]banktypes.SendEnabled{},
	)
	genesisState[banktypes.ModuleName] = appInstance.AppCodec().MustMarshalJSON(bankGenesis)

	_, err := tmtypes.PB2TM.ValidatorUpdates(initValPowers)
	if err != nil {
		panic("failed to get vals")
	}

	return genesisState, secp256k1.PrivKey{Key: privVal.PrivKey.Bytes()}
}

type TestEnv struct {
	App                *app.ArchwayApp
	Ctx                sdk.Context
	ParamTypesRegistry ParamTypeRegistry
	ValPrivs           []*secp256k1.PrivKey
	NodeHome           string
}

// DebugAppOptions is a stub implementing AppOptions
type DebugAppOptions struct{}

// Get implements AppOptions
func (ao DebugAppOptions) Get(o string) interface{} {
	if o == server.FlagTrace {
		return true
	}

	if o == "wasm.simulation_gas_limit" {
		return ^uint64(0) // max uint64
	}
	return nil
}

func NewArchwayApp(nodeHome string) *app.ArchwayApp {
	db := dbm.NewMemDB()

	return app.NewArchwayApp(
		log.NewNopLogger(),
		db,
		nil,
		true,
		map[int64]bool{},
		nodeHome,
		5,
		app.MakeEncodingConfig(),
		DebugAppOptions{},
		[]wasmdKeeper.Option{},
		baseapp.SetChainID("archway-1"),
	)
}

func InitChain(appInstance *app.ArchwayApp) (sdk.Context, secp256k1.PrivKey) {
	sdk.DefaultBondDenom = "aarch"
	genesisState, valPriv := GenesisStateWithValSet(appInstance)

	encCfg := app.MakeEncodingConfig()

	// Set up Wasm genesis state
	wasmGen := wasmtypes.GenesisState{
		Params: wasmtypes.Params{
			// Allow store code without gov
			CodeUploadAccess:             wasmtypes.AllowEverybody,
			InstantiateDefaultPermission: wasmtypes.AccessTypeEverybody,
		},
	}
	genesisState[wasmtypes.ModuleName] = encCfg.Marshaler.MustMarshalJSON(&wasmGen)

	// set staking genesis state
	stakingGenesisState := stakingtypes.GenesisState{}
	appInstance.AppCodec().UnmarshalJSON(genesisState[stakingtypes.ModuleName], &stakingGenesisState)

	stateBytes, err := json.MarshalIndent(genesisState, "", " ")

	requireNoErr(err)

	concensusParams := simtestutil.DefaultConsensusParams
	concensusParams.Block = &cmtproto.BlockParams{
		MaxBytes: 22020096,
		MaxGas:   300000000,
	}

	// replace sdk.DefaultDenom with "aarch", a bit of a hack, needs improvement
	stateBytes = []byte(strings.Replace(string(stateBytes), "\"stake\"", "\"aarch\"", -1))

	_, err = appInstance.InitChain(
		&abci.RequestInitChain{
			Validators:      []abci.ValidatorUpdate{},
			ConsensusParams: concensusParams,
			AppStateBytes:   stateBytes,
			ChainId:         "archway-1",
		},
	)
	if err != nil {
		panic(err)
	}

	ctx := appInstance.NewContextLegacy(false, cmtproto.Header{Height: 0, ChainID: "archway-1", Time: time.Now().UTC()})

	// for each stakingGenesisState.Validators
	for _, validator := range stakingGenesisState.Validators {
		consAddr, err := validator.GetConsAddr()
		requireNoErr(err)
		signingInfo := slashingtypes.NewValidatorSigningInfo(
			consAddr,
			ctx.BlockHeight(),
			0,
			time.Unix(0, 0),
			false,
			0,
		)
		err = appInstance.Keepers.SlashingKeeper.SetValidatorSigningInfo(ctx, consAddr, signingInfo)
		if err != nil {
			panic(err)
		}
	}

	return ctx, valPriv
}

func (env *TestEnv) BeginNewBlock(executeNextEpoch bool, timeIncreaseSeconds uint64) {
	validators, err := env.App.Keepers.StakingKeeper.GetAllValidators(env.Ctx)
	requireNoErr(err)
	valAddr, err := validators[0].GetConsAddr()
	requireNoErr(err)

	env.beginNewBlockWithProposer(executeNextEpoch, valAddr, timeIncreaseSeconds)
}

func (env *TestEnv) FundValidators() {
	for _, valPriv := range env.ValPrivs {
		valAddr := sdk.AccAddress(valPriv.PubKey().Address())
		err := banktestutil.FundAccount(env.Ctx, env.App.Keepers.BankKeeper, valAddr.Bytes(), sdk.NewCoins(sdk.NewInt64Coin("aarch", 9223372036854775807)))
		if err != nil {
			panic(errors.Wrapf(err, "Failed to fund account"))
		}
	}
}

func (env *TestEnv) GetValidatorAddresses() []string {
	validators, _ := env.App.Keepers.StakingKeeper.GetAllValidators(env.Ctx)
	var addresses []string
	for _, validator := range validators {
		addresses = append(addresses, validator.OperatorAddress)
	}

	return addresses
}

// beginNewBlockWithProposer begins a new block with a proposer.
func (env *TestEnv) beginNewBlockWithProposer(executeNextEpoch bool, proposer sdk.ValAddress, timeIncreaseSeconds uint64) {
	validator, err := env.App.Keepers.StakingKeeper.GetValidator(env.Ctx, proposer)
	requireNoErr(err)

	valConsAddr, err := validator.GetConsAddr()
	requireNoErr(err)

	valAddr := valConsAddr

	newBlockTime := env.Ctx.BlockTime().Add(time.Duration(timeIncreaseSeconds) * time.Second)

	header := cmtproto.Header{Height: env.Ctx.BlockHeight() + 1, Time: newBlockTime}
	env.Ctx = env.Ctx.WithBlockTime(newBlockTime).WithBlockHeight(env.Ctx.BlockHeight() + 1)
	voteInfos := []abci.VoteInfo{{
		Validator:   abci.Validator{Address: valAddr, Power: 1000},
		BlockIdFlag: cmtproto.BlockIDFlagCommit,
	}}
	env.Ctx = env.Ctx.WithVoteInfos(voteInfos)

	_, err = env.App.BeginBlocker(env.Ctx)
	requireNoErr(err)

	env.Ctx = env.App.NewContextLegacy(false, header)
}

func (env *TestEnv) SetupParamTypes() {
	pReg := env.ParamTypesRegistry

	pReg.RegisterParamSet(&rewards.Params{})
}

func requireNoErr(err error) {
	if err != nil {
		panic(err)
	}
}

func requireNoNil(name string, nilable any) {
	if nilable == nil {
		panic(fmt.Sprintf("%s must not be nil", name))
	}
}

func requierTrue(name string, b bool) {
	if !b {
		panic(fmt.Sprintf("%s must be true", name))
	}
}
