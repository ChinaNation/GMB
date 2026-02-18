import { useState } from 'react';
import {
  Accordion,
  AccordionDetails,
  AccordionSummary,
  Button,
  Stack,
  TextField,
  Typography
} from '@mui/material';
import { useSessionStore } from '../../stores/session';

export function DeveloperSettingsCard() {
  const endpoint = useSessionStore((state) => state.endpoint);
  const setEndpoint = useSessionStore((state) => state.setEndpoint);
  const [draft, setDraft] = useState(endpoint);

  return (
    <Accordion sx={{ backgroundColor: 'rgba(255,255,255,0.03)' }}>
      <AccordionSummary expandIcon={<Typography variant="body2">+</Typography>}>
        <Typography variant="body2">开发设置（RPC 地址）</Typography>
      </AccordionSummary>
      <AccordionDetails>
        <Stack direction={{ xs: 'column', md: 'row' }} spacing={1.5}>
          <TextField
            label="RPC Endpoint"
            value={draft}
            onChange={(event) => setDraft(event.target.value)}
            size="small"
            fullWidth
          />
          <Button
            variant="outlined"
            onClick={() => {
              const next = draft.trim();
              if (next) setEndpoint(next);
            }}
          >
            应用
          </Button>
        </Stack>
      </AccordionDetails>
    </Accordion>
  );
}
