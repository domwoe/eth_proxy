<template>
  <v-row justify="center" align="center">
    <v-col cols="12" sm="8" md="6">
      <v-card class="logo py-4 d-flex justify-center"> </v-card>
      <v-card>
        <v-card-title class="headline">
          <h3>MetaMask Internet Computer Demo</h3>
        </v-card-title>
        <v-card-text>
          <v-stepper v-model="step" vertical non-linear>
            <v-stepper-step :complete="step > 1" step="1">
              <h3>Connect Wallet</h3>
              <!-- <small>Summarize if needed</small> -->
            </v-stepper-step>

            <v-stepper-content step="1">
              <v-card align="center" justify="center" height="200px">
                <v-container fill-height fluid>
                  <v-card-text class="text-center">
                    <v-btn
                      v-if="!account"
                      color="primary"
                      @click="connectWallet"
                    >
                      Connect
                    </v-btn>
                    <h3 v-if="account">Wallet connected</h3>
                    <p v-if="account">Address: {{ account }}</p>
                    <p v-if="balance">Balance: {{ balance }}</p>
                    <v-btn v-if="account" color="primary" @click="step = 2">
                      Continue
                    </v-btn>
                  </v-card-text>
                </v-container>
              </v-card>
            </v-stepper-content>

            <v-stepper-step :complete="step > 2" step="2">
              <h3>Get testnet ETH</h3>
            </v-stepper-step>

            <v-stepper-content step="2">
              <v-card align="center" justify="center">
                <v-container fill-height fluid>
                  <v-card-text class="text-center">
                    <p>
                      Head over to
                      <a href="https://goerlifaucet.com" target="_blank"
                        >goerlifaucet.com</a
                      >
                      to get some testnet ETH in your wallet.
                    </p>

                    <v-text-field
                      :value="account"
                      small
                      outlined
                      readonly
                      append-icon="mdi-content-copy"
                      @click:append="copyAddress"
                    >
                    </v-text-field>

                    <v-btn v-if="account" color="primary" @click="step = 3">
                      Continue
                    </v-btn>
                  </v-card-text>
                </v-container>
              </v-card>
            </v-stepper-content>

            <v-stepper-step :complete="step > 3" step="3">
              Create proxy canister
            </v-stepper-step>

            <v-stepper-content step="3">
              {{ proxy }}

              <v-btn color="primary" @click="step = 4"> Continue </v-btn>
              <v-btn text> Cancel </v-btn>
            </v-stepper-content>

            <v-stepper-step :complete="step > 4" step="4">
              <h3>Swap ETH for ckETH</h3>
            </v-stepper-step>
            <v-stepper-content step="3">
              <p v-if="balance">Balance: {{ balance }}</p>

              <v-btn color="primary" @click="swap" :loading="isSending">
                Swap 0.001 ETH
              </v-btn>

              <v-btn color="tertiary" @click="step = 5"> Continue </v-btn>
            </v-stepper-content>

            <v-stepper-step :complete="step > 5" step="5">
              <h3>Connect to proxy canister</h3>
            </v-stepper-step>
            <v-stepper-content step="5">
              <v-btn color="primary" @click="addIC">
                Connect to Internet Computer
              </v-btn>
            </v-stepper-content>

            <v-stepper-step :complete="step > 6" step="6">
              <h3>Add ckETH to wallet</h3>
            </v-stepper-step>
            <v-stepper-content step="6">
              <v-btn color="primary" @click="addCkETH">
                Add ckETH to wallet
              </v-btn>
            </v-stepper-content>

            <v-stepper-step :complete="step > 7" step="7">
              <h3>Send ckETH</h3>
            </v-stepper-step>
            <v-stepper-content step="7">
              <p>You can now</p>
            </v-stepper-content>
          </v-stepper>
        </v-card-text>
      </v-card>
    </v-col>
  </v-row>
</template>

<script>
import { Buffer } from 'buffer'
// eslint-disable-next-line
import { proxy_manager } from '../.dfx/local/canisters/proxy_manager'
export default {
  name: 'IndexPage',
  data() {
    return {
      step: 1,
      isConnected: false,
      account: null,
      chainId: null,
      balance: null,
      isSending: false,
    }
  },
  mounted() {
    window.global = window
    window.Buffer = Buffer
    this.chainId = window.ethereum.networkVersion

    window.ethereum.on('connected', () => {
      console.log('Connected!')
      this.isConnected = true
    })

    window.ethereum.on('accountsChanged', (accounts) => {
      console.log('Account: ' + accounts[0])
      this.account = accounts[0]
    })

    window.ethereum.on('networkChanged', (networkId) => {
      console.log('Network: ' + networkId)
      this.chainId = networkId
      if (networkId === '255') {
        this.step++
      }
    })
  },
  methods: {
    copyAddress() {
      navigator.clipboard.writeText(this.account)
    },
    async connectWallet() {
      try {
        const accounts = await window.ethereum.request({
          method: 'eth_requestAccounts',
        })

        this.account = accounts[0]

        if (window.ethereum.networkVersion !== '0x5') {
          this.chainId = await window.ethereum.request({
            method: 'wallet_switchEthereumChain',
            params: [{ chainId: '0x5' }],
          })
        }

        await this.getProxy()

        this.balance =
          Math.round(
            parseInt(
              await window.ethereum.request({
                method: 'eth_getBalance',
                params: [this.account, 'latest'],
              })
            ) *
              10 ** -16
          ) / 100
      } catch (e) {
        console.error(e)
      }
    },
    async getProxy() {
      // eslint-disable-next-line
      this.proxy = await proxy_manager.get_proxy(this.account)
    },
    async swap() {
      this.isSending = true
      const amount = 0.001 * 10 ** 18
      const amountHex = amount.toString(16).toUpperCase()

      const transactionParameters = {
        to: '0x0000000000000000000000000000000000000000',
        from: this.account,
        value: amountHex,
        chainId: '0x5',
      }
      let txHash = null
      try {
        txHash = await window.ethereum.request({
          method: 'eth_sendTransaction',
          params: [transactionParameters],
        })
      } catch (e) {
        console.error(e)
      }

      this.isSending = false

      if (txHash) {
        this.step = 4
      }
    },
    async addIC() {
      let success = false
      try {
        success = await window.ethereum.request({
          method: 'wallet_switchEthereumChain',
          params: [{ chainId: '0x255' }],
        })
      } catch (error) {
        if (error.code === 4902) {
          try {
            await window.ethereum.request({
              method: 'wallet_addEthereumChain',
              params: [
                {
                  chainId: '0x255',
                  chainName: 'Internet Computer',
                  nativeCurrency: {
                    name: 'ICP',
                    symbol: 'ICP',
                    decimals: 8,
                  },
                  rpcUrls: [
                    // 'http://127.0.0.1:4943/?canisterId=rrkah-fqaaa-aaaaa-aaaaq-cai',
                    'https://ezw7y-fqaaa-aaaap-qatua-cai.raw.ic0.app',
                  ],
                  blockExplorerUrls: [],
                  iconUrls: [
                    'https://s2.coinmarketcap.com/static/img/coins/64x64/8916.png',
                  ],
                },
              ],
            })
          } catch (addError) {
            console.log('Did not add network')
          }
        }
      }

      if (success) {
        this.step = 5
      }
    },
    async addCkETH() {
      try {
        const success = await window.ethereum.request({
          method: 'wallet_watchAsset',
          params: {
            type: 'ERC20', // Initially only supports ERC20, but eventually more!
            options: {
              address: '0x71c7656ec7ab88b098defb751b7401b5f6d89765',
              symbol: 'ckETH',
              decimals: 13,
              image:
                'https://s2.coinmarketcap.com/static/img/coins/200x200/1027.png',
            },
          },
        })

        if (!success) {
          console.log('ckETH not added')
        }
      } catch (error) {
        console.log(error)
      }
    },
  },
}
</script>
