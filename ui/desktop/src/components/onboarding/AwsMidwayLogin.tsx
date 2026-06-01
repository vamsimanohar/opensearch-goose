import { useEffect, useState } from 'react';
import { Badge, AlertTriangle, CheckCircle2, Loader2 } from 'lucide-react';
import { Button } from '../ui/button';
import { awsMidwayLogin, awsMidwayStatus } from '../../api';

interface AwsMidwayLoginProps {
  onConfigured: (providerName: string) => void;
}

type Phase = 'checking' | 'no-ada' | 'input' | 'authenticating' | 'ok' | 'error';
type Provider = 'isengard' | 'conduit';

const REGIONS = [
  'us-west-2',
  'us-east-1',
  'us-east-2',
  'eu-west-1',
  'eu-central-1',
  'ap-northeast-1',
];

export default function AwsMidwayLogin({ onConfigured }: AwsMidwayLoginProps) {
  const [phase, setPhase] = useState<Phase>('checking');
  const [adaMessage, setAdaMessage] = useState('');

  const [accountId, setAccountId] = useState('');
  const [roleName, setRoleName] = useState('Admin');
  const [provider, setProvider] = useState<Provider>('isengard');
  const [region, setRegion] = useState('us-west-2');
  const [profileName, setProfileName] = useState('goose-midway');

  const [identity, setIdentity] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Detect ada at mount.
  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const resp = await awsMidwayStatus({ throwOnError: true });
        if (cancelled) return;
        if (resp.data?.ada_installed) {
          setPhase('input');
        } else {
          setAdaMessage(resp.data?.message ?? '`ada` not found.');
          setPhase('no-ada');
        }
      } catch (e) {
        if (cancelled) return;
        setAdaMessage(e instanceof Error ? e.message : String(e));
        setPhase('no-ada');
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  const handleSignIn = async () => {
    if (!/^\d{12}$/.test(accountId.trim())) {
      setError('AWS account id must be 12 digits.');
      return;
    }
    if (!roleName.trim()) {
      setError('Enter a role name.');
      return;
    }
    setError(null);
    setPhase('authenticating');
    try {
      const resp = await awsMidwayLogin({
        body: {
          account_id: accountId.trim(),
          role_name: roleName.trim(),
          provider,
          region,
          profile_name: profileName.trim() || null,
        },
        throwOnError: true,
      });
      const data = resp.data;
      if (data?.success) {
        setIdentity(data.identity ?? `${accountId}/${roleName}`);
        setPhase('ok');
      } else {
        setError(data?.message ?? 'Login failed.');
        setPhase('error');
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setPhase('error');
    }
  };

  return (
    <div className="flex flex-col gap-4 max-w-lg mx-auto">
      <div className="flex items-center gap-3">
        <Badge className="w-8 h-8 text-amber-500" />
        <div>
          <h2 className="text-xl font-semibold">Sign in with Midway</h2>
          <p className="text-sm text-text-muted">
            For Amazon employees. Uses the <code>ada</code> CLI to fetch temporary
            credentials from Isengard or Conduit. Make sure you've run{' '}
            <code>mwinit -o</code> first.
          </p>
        </div>
      </div>

      {phase === 'checking' && (
        <div className="flex items-center gap-2 text-sm text-text-muted">
          <Loader2 className="animate-spin" size={16} /> Checking for `ada`…
        </div>
      )}

      {phase === 'no-ada' && (
        <div className="flex flex-col gap-3 border border-amber-500/30 rounded-lg p-3">
          <div className="flex items-start gap-2 text-sm">
            <AlertTriangle size={16} className="text-amber-500 mt-0.5" />
            <div>
              <p className="font-medium text-amber-500">`ada` not found</p>
              <p className="text-text-muted">{adaMessage}</p>
              <p className="text-text-muted mt-2">
                Install it with: <code>toolbox install ada</code>
              </p>
            </div>
          </div>
        </div>
      )}

      {(phase === 'input' || phase === 'authenticating' || phase === 'error') && (
        <>
          <div className="grid grid-cols-2 gap-3">
            <div className="flex flex-col gap-1 col-span-2">
              <label className="text-sm font-medium">AWS Account ID</label>
              <input
                type="text"
                inputMode="numeric"
                pattern="\d{12}"
                value={accountId}
                onChange={(e) => setAccountId(e.target.value.replace(/\D/g, '').slice(0, 12))}
                placeholder="123456789012"
                className="px-3 py-2 border rounded-lg bg-transparent"
                disabled={phase === 'authenticating'}
                autoFocus
              />
            </div>

            <div className="flex flex-col gap-1">
              <label className="text-sm font-medium">Role</label>
              <input
                type="text"
                value={roleName}
                onChange={(e) => setRoleName(e.target.value)}
                placeholder="Admin"
                className="px-3 py-2 border rounded-lg bg-transparent"
                disabled={phase === 'authenticating'}
              />
            </div>

            <div className="flex flex-col gap-1">
              <label className="text-sm font-medium">Provider</label>
              <select
                value={provider}
                onChange={(e) => setProvider(e.target.value as Provider)}
                className="px-3 py-2 border rounded-lg bg-transparent"
                disabled={phase === 'authenticating'}
              >
                <option value="isengard">Isengard</option>
                <option value="conduit">Conduit</option>
              </select>
            </div>

            <div className="flex flex-col gap-1">
              <label className="text-sm font-medium">Region</label>
              <select
                value={region}
                onChange={(e) => setRegion(e.target.value)}
                className="px-3 py-2 border rounded-lg bg-transparent"
                disabled={phase === 'authenticating'}
              >
                {REGIONS.map((r) => (
                  <option key={r} value={r}>
                    {r}
                  </option>
                ))}
              </select>
            </div>

            <div className="flex flex-col gap-1">
              <label className="text-sm font-medium">Profile name</label>
              <input
                type="text"
                value={profileName}
                onChange={(e) => setProfileName(e.target.value)}
                className="px-3 py-2 border rounded-lg bg-transparent"
                disabled={phase === 'authenticating'}
              />
              <span className="text-xs text-text-muted">
                Written to <code>~/.aws/credentials</code>
              </span>
            </div>
          </div>

          {error && phase === 'error' && (
            <div className="text-sm text-red-500 border border-red-500/30 rounded-lg p-3 whitespace-pre-wrap">
              {error}
            </div>
          )}

          <Button
            onClick={handleSignIn}
            size="lg"
            className="self-start"
            disabled={phase === 'authenticating'}
          >
            {phase === 'authenticating' ? (
              <>
                <Loader2 className="animate-spin mr-2" size={16} />
                Verifying with ada…
              </>
            ) : (
              'Sign in'
            )}
          </Button>
        </>
      )}

      {phase === 'ok' && identity && (
        <div className="flex flex-col gap-3">
          <div className="flex items-start gap-2 text-sm border border-green-500/30 rounded-lg p-3">
            <CheckCircle2 size={16} className="text-green-500 mt-0.5" />
            <div>
              <p className="font-medium text-green-500">Signed in</p>
              <p className="text-text-muted">{identity}</p>
              <p className="text-text-muted mt-1">
                Profile <code>{profileName}</code> will refresh credentials automatically
                via <code>ada</code>.
              </p>
            </div>
          </div>
          <Button onClick={() => onConfigured('aws_bedrock')} size="lg" className="self-start">
            Continue
          </Button>
        </div>
      )}
    </div>
  );
}
