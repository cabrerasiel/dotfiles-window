import { useEffect, useState } from 'react';
import { createProviderGroup, type ProviderGroup } from 'zebar';
import { Button } from '@/components/ui/button';

type ProviderMap = ProviderGroup<{
  glazewm: { type: 'glazewm' };
  date: { type: 'date'; formatting: string };
  media: { type: 'media' };
}>;

const providers = createProviderGroup({
  glazewm: { type: 'glazewm' },
  date: { type: 'date', formatting: 'EEE d MMM HH:mm' },
  media: { type: 'media' },
});

function App() {
  const [output, setOutput] = useState<ProviderMap['outputMap'] | null>(null);

  useEffect(() => {
    providers.onOutput(setOutput);
  }, []);

  const glazewm = output?.glazewm;
  const date = output?.date;
  const media = output?.media;

  return (
    <div className="grid grid-cols-3 items-center h-full px-4 text-sm text-white/90">
      <div className="flex items-center gap-1">
        {glazewm?.currentWorkspaces?.map(ws => (
          <Button
            key={ws.name}
            variant={ws.hasFocus ? 'default' : ws.isDisplayed ? 'displayed' : 'ghost'}
            className="h-6 px-2 text-xs"
            onClick={() => glazewm.runCommand(`focus --workspace ${ws.name}`)}
          >
            {ws.displayName ?? ws.name}
          </Button>
        ))}
      </div>

      <div className="justify-self-center flex items-center gap-2 text-white/40 text-xs">
        {media?.currentSession && (
          <>
            <span>{media.currentSession.artist}</span>
            <span className="text-white/20">—</span>
            <span>{media.currentSession.title}</span>
          </>
        )}
      </div>

      <div className="justify-self-end">
        {date?.formatted ?? ''}
      </div>
    </div>
  );
}

export default App;
