import { useState, useEffect } from 'react';
import { connect, Contract, keyStores, WalletConnection } from 'near-api-js';

interface TwitterLinkingContract extends Contract {
  init_linking: () => Promise<string>;
  verify_link: (args: { linking_id: string, oauth_token: string }) => Promise<void>;
  get_twitter_link: (args: { account_id: string }) => Promise<{ handle: string; user_id: string } | null>;
}

const nearConfig = {
  networkId: 'testnet',
  nodeUrl: 'https://rpc.testnet.near.org',
  walletUrl: 'https://wallet.testnet.near.org',
  helperUrl: 'https://helper.testnet.near.org',
  contractName: 'social-linking.testnet',
};

const LinkTwitterAccount: React.FC = () => {
  const [linkingId, setLinkingId] = useState<string>();
  const [walletConnection, setWalletConnection] = useState<WalletConnection>();
  const [contract, setContract] = useState<TwitterLinkingContract>();
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    initNear().catch(console.error);
  }, []);

  const initNear = async () => {
    const near = await connect({
      ...nearConfig,
      keyStore: new keyStores.BrowserLocalStorageKeyStore(),
    });

    const walletConnection = new WalletConnection(near, 'twitter-linking');
    setWalletConnection(walletConnection);

    if (walletConnection.isSignedIn()) {
      const contract = new Contract(
        walletConnection.account(),
        nearConfig.contractName,
        {
          viewMethods: ['get_twitter_link'],
          changeMethods: ['init_linking', 'verify_link'],
          useLocalViewExecution: false
        }
      ) as TwitterLinkingContract;
      setContract(contract);
    }
  };

  const handleSignIn = () => {
    walletConnection?.requestSignIn({
      contractId: nearConfig.contractName,
      methodNames: ['init_linking', 'verify_link'],
      keyType: 'ed25519'
    });
  };

  const generateTwitterOAuthUrl = (linkingId: string) => {
    const clientId = import.meta.env.VITE_TWITTER_CLIENT_ID;
    const redirectUri = encodeURIComponent(window.location.origin + '/callback');
    const state = encodeURIComponent(linkingId);
    
    return `https://twitter.com/i/oauth2/authorize?client_id=${clientId}&redirect_uri=${redirectUri}&state=${state}&response_type=code&scope=tweet.read%20users.read`;
  };

  const initiateLinking = async () => {
    if (!contract) return;
    
    try {
      setLoading(true);
      // Get linking_id from contract
      const id = await contract.init_linking();
      setLinkingId(id);

      // Redirect to Twitter OAuth
      const oauthUrl = generateTwitterOAuthUrl(id);
      window.location.href = oauthUrl;
    } catch (error) {
      console.error('Error initiating linking:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleOAuthCallback = async () => {
    if (!contract || !linkingId) return;

    const params = new URLSearchParams(window.location.search);
    const code = params.get('code');
    const state = params.get('state');

    // Verify state matches linkingId
    if (state !== linkingId) {
      console.error('Invalid state parameter');
      return;
    }

    if (code) {
      try {
        setLoading(true);
        await contract.verify_link({ linking_id: linkingId, oauth_token: code });
      } catch (error) {
        console.error('Error verifying link:', error);
      } finally {
        setLoading(false);
      }
    }
  };

  useEffect(() => {
    // Check if we're on the callback route
    if (window.location.pathname === '/callback') {
      handleOAuthCallback();
    }
  }, [contract, linkingId]);

  if (!walletConnection?.isSignedIn()) {
    return (
      <div className="flex justify-center items-center min-h-screen">
        <button
          onClick={handleSignIn}
          className="bg-blue-500 hover:bg-blue-600 text-white font-bold py-2 px-4 rounded"
        >
          Connect NEAR Wallet
        </button>
      </div>
    );
  }

  return (
    <div className="flex flex-col items-center justify-center min-h-screen p-4">
      <h1 className="text-2xl font-bold mb-8">Link Your Twitter Account</h1>
      
      <button
        onClick={initiateLinking}
        disabled={loading}
        className={`
          bg-blue-500 hover:bg-blue-600 text-white font-bold py-2 px-4 rounded
          ${loading ? 'opacity-50 cursor-not-allowed' : ''}
        `}
      >
        {loading ? 'Processing...' : 'Link Twitter Account'}
      </button>
    </div>
  );
};

export default LinkTwitterAccount;
