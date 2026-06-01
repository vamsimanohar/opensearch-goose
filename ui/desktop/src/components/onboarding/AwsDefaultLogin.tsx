import { useState } from 'react';
import { CheckCircle2, Terminal, Loader2 } from 'lucide-react';
import { Button } from '../ui/button';
import { awsDefaultProbe } from '../../api';

interface AwsDefaultLoginProps {
  onConfigured: (providerName: string) => void;
}

type Phase = 'idle' | 'probing' | 'ok' | 'error';

export default function AwsDefaultLogin({ onConfigured }: AwsDefaultLoginProps) {
  const [phase, setPhase] = useState<Phase>('idle');
  const [identity, setIdentity] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const probe = async () => {
    setPhase('probing');
    setError(null);
    try {
      const resp = await awsDefaultProbe({ throwOnError: true });
      const data = resp.data;
      if (data?.success) {
        setIdentity(data.identity ?? null);
        setPhase('ok');
      } else {
        setError(data?.message ?? 'Could not resolve AWS credentials.');
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
        <Terminal className="w-8 h-8 text-text-muted" />
        <div>
          <h2 className="text-xl font-semibold">Use existing AWS credentials</h2>
          <p className="text-sm text-text-muted">
            Goose will use whatever credentials your AWS SDK default chain finds: env
            vars, <code>~/.aws/credentials</code>, an active SSO session, or your IAM
            instance role.
          </p>
        </div>
      </div>

      {phase === 'idle' && (
        <Button onClick={probe} size="lg" className="self-start">
          Verify and continue
        </Button>
      )}

      {phase === 'probing' && (
        <div className="flex items-center gap-2 text-sm text-text-muted">
          <Loader2 className="animate-spin" size={16} />
          Checking AWS credentials…
        </div>
      )}

      {phase === 'ok' && identity && (
        <div className="flex flex-col gap-3">
          <div className="flex items-start gap-2 text-sm border border-green-500/30 rounded-lg p-3">
            <CheckCircle2 size={16} className="text-green-500 mt-0.5" />
            <div>
              <p className="font-medium text-green-500">Credentials resolved</p>
              <p className="text-text-muted break-all">{identity}</p>
            </div>
          </div>
          <Button onClick={() => onConfigured('aws_bedrock')} size="lg" className="self-start">
            Continue
          </Button>
        </div>
      )}

      {phase === 'error' && (
        <div className="flex flex-col gap-3">
          <div className="text-sm text-red-500 border border-red-500/30 rounded-lg p-3 whitespace-pre-wrap">
            {error}
          </div>
          <Button onClick={probe} variant="outline">
            Try again
          </Button>
        </div>
      )}
    </div>
  );
}
