import type { ThemeConfig } from 'antd';

export const antdTheme: ThemeConfig = {
  token: {
    colorPrimary: '#1d6e6b',
    colorInfo: '#1d6e6b',
    colorSuccess: '#2f8f5b',
    colorWarning: '#b27a23',
    colorError: '#c24b5a',
    borderRadius: 18,
    fontFamily: '"Noto Sans SC", "PingFang SC", "Microsoft YaHei", sans-serif'
  },
  components: {
    Card: {
      borderRadiusLG: 24
    },
    Input: {
      borderRadiusLG: 16
    },
    Button: {
      borderRadiusLG: 24,
      controlHeightLG: 48,
      fontWeight: 700
    },
    Alert: {
      borderRadiusLG: 16
    }
  }
};
