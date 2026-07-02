import * as React from 'react';
import { cn } from '@/lib/utils';

const buttonVariants = {
  default:
    'bg-[rgba(75,115,255,0.6)] text-white/90 shadow-xs hover:bg-[rgba(75,115,255,0.7)]',
  displayed:
    'bg-white/12 text-white/90 hover:bg-white/18',
  ghost:
    'bg-white/6 text-white/70 hover:bg-white/12 hover:text-white/90',
};

type ButtonProps = React.ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: keyof typeof buttonVariants;
};

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant = 'ghost', ...props }, ref) => {
    return (
      <button
        className={cn(
          'inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-sm text-sm font-medium transition-colors focus-visible:outline-hidden disabled:pointer-events-none disabled:opacity-50',
          'px-1.5 py-0.5',
          buttonVariants[variant],
          className,
        )}
        ref={ref}
        {...props}
      />
    );
  },
);
Button.displayName = 'Button';

export { Button };
